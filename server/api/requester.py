import datetime
import logging
import uuid
from typing import TypedDict, cast
from urllib.parse import urlparse

import requests
from django.db import transaction
from pyld import jsonld

from api.models import (
    Post,
    PublicKey,
    RemoteUserInfo,
    User,
    UserFollow,
    UserFollowRequest,
)
from api.serializers.pub import PUBLIC_URI, Actor, Note, is_actor
from api.utils.get_id import (
    get_post_id,
    get_user_id,
    get_user_public_key_id,
    is_local_uri,
    make_followers_id,
)
from api.utils.signature import attach_signature
from lightpub.settings import DEBUG

ssl_verify = not DEBUG

HEADERS = {
    "accept": "application/activity+json",
}

logger = logging.getLogger(__name__)


class RequesterException(Exception):
    pass


class NotRemoteError(RequesterException):
    def __init__(self, uri: str, msg: str) -> None:
        super().__init__(msg)
        self.uri = uri
        self.msg = msg


class RemoteRelatedError(RequesterException):
    pass


class RemoteObjectNotFoundError(RemoteRelatedError):
    def __init__(self, uri: str) -> None:
        self.uri = uri


class MalformedRemoteResponseError(RemoteRelatedError):
    def __init__(self, uri: str, msg: str) -> None:
        self.uri = uri
        self.msg = msg


class Requester:
    def __init__(self) -> None:
        self._session = requests.Session()
        self.default_timeout = 3
        logger.info("Requester initialized")

    def fetch_remote_id(self, id: str):
        if is_local_uri(id):
            raise NotRemoteError(id, "not a remote id")
        logger.debug("fetching remote resource: %s", id)
        res = self._session.get(
            id, verify=ssl_verify, headers=HEADERS, timeout=self.default_timeout
        )
        try:
            res.raise_for_status()
        except requests.exceptions.RequestException as e:
            if e.response.status_code == 404:
                raise RemoteObjectNotFoundError(id)
            raise e
        j = res.json()
        return jsonld.expand(j)

    def fetch_remote_user(self, uri: str, force_fetch: bool = False):
        # TODO: respect force_fetch, use db if possible
        host = urlparse(uri).hostname
        e = self.fetch_remote_id(uri)

        actor = Actor.from_dict(e[0])
        if not is_actor(actor):
            raise ValueError("not an actor, type is " + str(actor.type))

        return _get_or_insert_remote_user(actor, host)

    def fetch_remote_post_by_uri(self, uri: str, nested: int = 0) -> uuid.UUID:
        # TODO: use nested to determine the content should be fetched
        e = self.fetch_remote_id(uri)

        if len(e) != 1:
            logger.debug("malformed response: %s", str(e))
            raise MalformedRemoteResponseError(uri, "expected 1 object")
        note = Note.from_dict(e[0])
        user = self.fetch_remote_user(note.as_attributedTo.id)
        post = Post(
            uri=note.id,
            poster=user,
            content=note.as_content,
            created_at=note.as_published.as_datetime(),
            privacy=0,  # TODO: implement privacy
            reply_to=None,  # TODO: implement reply_to
        )
        post.save()

        return post.id

    def _get_actor_id_from_username(self, username: str, host: str) -> str:
        webfinger_url = f"https://{host}/.well-known/webfinger"
        webfinger_params = {
            "resource": f"acct:{username}",
        }
        res = self._session.get(
            webfinger_url,
            params=webfinger_params,
            verify=ssl_verify,
            timeout=self.default_timeout,
        )
        res.raise_for_status()
        j = res.json()
        links = j["links"]
        for link in links:
            if link["rel"] == "self" and link["type"] == "application/activity+json":
                return link["href"]

        raise ValueError("actor not found")

    def fetch_remote_username(self, username: str, host: str) -> uuid.UUID:
        id = self._get_actor_id_from_username(username, host)
        res = self.fetch_remote_id(id)

        actor = Actor.from_dict(res[0])
        if not is_actor(actor):
            raise ValueError("not an actor, type is " + str(actor.type))

        return _get_or_insert_remote_user(actor, host)

    def send_follow_accept(self, follow_req: UserFollowRequest) -> None:
        inbox_url = follow_req.follower.inbox
        if inbox_url is None:
            raise ValueError("inbox url is not set")

        if follow_req.uri is None:
            raise ValueError("follow request url is not set")

        # prepare private key
        sender = follow_req.followee
        sender_id = get_user_id(sender)
        private_key = sender.private_key
        if not private_key:
            raise ValueError("private key is not set")
        key_id = get_user_public_key_id(sender)

        req = requests.Request(
            method="POST",
            url=inbox_url,
            json={
                "@context": [
                    "https://www.w3.org/ns/activitystreams",
                ],
                "type": "Accept",
                "object": follow_req.uri,
                "actor": sender_id,
            },
            headers={
                "Content-Type": "application/activity+json",
            },
        )
        prep = req.prepare()
        attach_signature(prep, key_id, private_key.encode("utf-8"))
        res = self._session.send(prep, timeout=self.default_timeout, verify=ssl_verify)
        res.raise_for_status()

        with transaction.atomic():
            # check if follow is already exists
            if not UserFollow.objects.filter(
                follower=follow_req.follower,
                followee=follow_req.followee,
            ).exists():
                # create a new user follow
                actual_follow = UserFollow(
                    follower=follow_req.follower,
                    followee=follow_req.followee,
                )
                actual_follow.save()

            # delete the follow request
            follow_req.delete()

    def send_unfollow(self, follower: User, followee: User) -> None:
        inbox_url = followee.inbox
        if inbox_url is None:
            raise ValueError("inbox url is not set")

        # prepare private key
        sender_id = get_user_id(follower)
        private_key = follower.private_key
        if not private_key:
            raise ValueError("private key is not set")
        key_id = get_user_public_key_id(follower)

        req = requests.Request(
            method="POST",
            url=inbox_url,
            json={
                "@context": [
                    "https://www.w3.org/ns/activitystreams",
                ],
                "type": "Undo",
                "object": {
                    "type": "Follow",
                    "actor": sender_id,
                    "object": get_user_id(followee),
                },
                "actor": sender_id,
            },
            headers={
                "Content-Type": "application/activity+json",
            },
        )
        prep = req.prepare()
        attach_signature(prep, key_id, private_key.encode("utf-8"))
        res = self._session.send(prep, timeout=self.default_timeout, verify=ssl_verify)
        res.raise_for_status()

    def send_follow_request(self, follow_req: UserFollowRequest) -> None:
        inbox = follow_req.followee.inbox
        if inbox is None:
            raise ValueError("followee's inbox url is not set")

        followee_uri = follow_req.followee.uri
        if followee_uri is None:
            raise ValueError("followee's uri is not set")

        follower_uri = follow_req.follower.uri
        # should not be set because it is a local user
        if follower_uri is not None:
            raise ValueError("follower's uri is set")
        follower_uri = get_user_id(follow_req.follower)

        req = requests.Request(
            method="POST",
            url=inbox,
            json={
                "@context": [
                    "https://www.w3.org/ns/activitystreams",
                ],
                "type": "Follow",
                "object": followee_uri,
                "actor": follower_uri,
            },
            headers={
                "Content-Type": "application/activity+json",
            },
        )
        prep = req.prepare()
        attach_signature(
            prep,
            get_user_public_key_id(follow_req.follower),
            follow_req.follower.private_key.encode("utf-8"),
        )
        res = self._session.send(
            prep,
            verify=ssl_verify,
            timeout=self.default_timeout,
        )
        res.raise_for_status()

    def send_post_to_federated_servers(self, post: Post) -> None:
        tocc = _make_to_and_cc(post)
        to = tocc["to"]
        cc = tocc["cc"]
        target_inboxes = tocc["target_inboxes"]

        # prepare private key
        sender = post.poster
        sender_id = get_user_id(sender)
        private_key = sender.private_key
        if not private_key:
            raise ValueError("private key is not set")
        key_id = get_user_public_key_id(sender)

        post_id = get_post_id(post)
        create_id = f"{post_id}activity/"  # TODO: really?

        # send to each inbox
        for inbox in target_inboxes:
            # TODO: ここらへんの処理ループ内でやる必要ある?
            req = requests.Request(
                method="POST",
                url=inbox,
                json={
                    "@context": [
                        "https://www.w3.org/ns/activitystreams",
                    ],
                    "id": create_id,
                    "type": "Create",
                    "actor": sender_id,
                    "to": to,
                    "cc": cc,
                    "object": {
                        "id": post_id,
                        "attributedTo": sender_id,
                        "type": "Note",
                        "to": cc,
                        "cc": cc,
                        "content": post.content,
                        "published": post.created_at.isoformat(),
                        "sensitive": False,
                    },
                    "published": post.created_at.isoformat(),
                },
                headers={
                    "Content-Type": "application/activity+json",
                },
            )
            prep = req.prepare()
            attach_signature(prep, key_id, private_key.encode("utf-8"))
            try:
                res = self._session.send(
                    prep, timeout=self.default_timeout, verify=ssl_verify
                )
                res.raise_for_status()
            except requests.exceptions.RequestException as e:
                print("failed to post a post to a remote inbox")
                print(e)


_req = Requester()


def get_requester() -> Requester:
    return _req


class PostToCcList(TypedDict):
    to: list[str]
    cc: list[str]
    target_inboxes: list[str]


def get_followers_inboxes(user: User) -> list[str]:
    followers = (
        UserFollow.objects.filter(followee=user, follower__host__isnull=False)
        .select_related("follower")
        .all()
    )

    # TODO: send to sharedInbox instead of individual inboxes
    inboxes = []
    for f in followers:
        if f.follower.inbox:
            inboxes.append(f.follower.inbox)

    return inboxes


def _make_to_and_cc(post: Post) -> PostToCcList:
    # check post's privacy
    # TODO: consider mentions
    if post.privacy == 0:
        # public
        to = [PUBLIC_URI]
        cc = [make_followers_id(post.poster)]
        target_inboxes = get_followers_inboxes(post.poster)
    elif post.privacy == 1:
        # unlisted
        to = [make_followers_id(post.poster)]
        cc = [PUBLIC_URI]
        target_inboxes = get_followers_inboxes(post.poster)
    elif post.privacy == 2:
        # followers only
        to = [make_followers_id(post.poster)]
        cc = []
        target_inboxes = get_followers_inboxes(post.poster)
    elif post.privacy == 3:
        # private only
        to = []
        cc = []
        target_inboxes = []
    else:
        raise ValueError("invalid privacy")

    return {
        "to": to,
        "cc": cc,
        "target_inboxes": target_inboxes,
    }


def get_or_insert_remote_user(actor: Actor, hostname: str) -> uuid.UUID:
    return _get_or_insert_remote_user(actor, hostname)


def _get_or_insert_remote_user(actor: Actor, hostname: str) -> uuid.UUID:
    # check if user already exists
    new_user: User | None = None
    remote_info: RemoteUserInfo | None = None
    user = User.objects.filter(uri=actor.id).first()
    if user is not None:
        remote_user_info = cast(RemoteUserInfo | None, user.remote_user_info)
        if remote_user_info is None:
            # user need to be updated
            new_user_ = user
            remote_info = None
        else:
            # TODO: parameterize the threshold
            if remote_user_info.last_fetched_at < datetime.datetime.now(
                datetime.timezone.utc
            ) - datetime.timedelta(days=1):
                # user need to be updated
                new_user_ = user
                remote_info = remote_user_info
            else:
                # user is up-to-date
                return user.id
        # update existing user
        new_user_.username = actor.as_preferred_username
        new_user_.nickname = actor.as_name
        new_user_.uri = actor.id  # TODO: uri update して大丈夫?
        new_user_.inbox = actor.as_inbox.id
        new_user_.outbox = actor.as_outbox.id
        new_user = new_user_
    else:
        # create a new remote user
        new_user = User(
            username=actor.as_preferred_username,
            host=hostname,
            bpassword=None,
            nickname=actor.as_name,
            uri=actor.id,
            inbox=actor.as_inbox.id,
            outbox=actor.as_outbox.id,
        )

    if remote_info is None:
        remote_info = RemoteUserInfo(
            user=new_user,
            following=actor.as_following.id if actor.as_following else None,
            followers=actor.as_followers.id if actor.as_followers else None,
            liked=actor.as_liked.id if actor.as_liked else None,
            shared_inbox=actor.as_shared_inbox.id if actor.as_shared_inbox else None,
            preferred_username=actor.as_preferred_username,
        )
    else:
        remote_info.following = actor.as_following.id if actor.as_following else None
        remote_info.followers = actor.as_followers.id if actor.as_followers else None
        remote_info.liked = actor.as_liked.id if actor.as_liked else None
        remote_info.preferred_username = actor.as_preferred_username
        remote_info.shared_inbox = (
            actor.as_shared_inbox.id if actor.as_shared_inbox else None
        )
        remote_info.last_fetched_at = datetime.datetime.now()

    public_key_info = None
    if actor.as_public_key:
        k = actor.as_public_key
        if not k.as_owner:
            raise ValueError("public key does not have owner")
        elif k.as_owner.id != actor.id:
            raise ValueError("public key owner does not match actor id")

        if not k.as_public_key_pem:
            raise ValueError("public key pem is not set")

        public_key_info = PublicKey(
            uri=k.id,
            user=new_user,
            public_key_pem=k.as_public_key_pem,
        )

    # use transaction to ensure consistency
    with transaction.atomic():
        new_user.save()
        remote_info.save()
        if public_key_info:
            public_key_info.save()

    return new_user.id
