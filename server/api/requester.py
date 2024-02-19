from pprint import pprint
import requests
from api.serializers.pub import Actor, is_actor
from api.models import User, RemoteUserInfo, UserFollow, UserFollowRequest
from pyld import jsonld
from urllib.parse import urlparse
from django.db import transaction
from lightpub.settings import DEBUG

ssl_verify = not DEBUG

HEADERS = {
    "accept": "application/activity+json",
}


class Requester:
    def __init__(self) -> None:
        self._session = requests.Session()
        self.default_timeout = 3

    def fetch_remote_id(self, id: str):
        res = self._session.get(
            id, verify=ssl_verify, headers=HEADERS, timeout=self.default_timeout
        )
        j = res.json()
        return jsonld.expand(j)

    def fetch_remote_user(self, id: str):
        host = urlparse(id).hostname
        e = self.fetch_remote_id(id)

        actor = Actor.from_dict(e[0])
        if not is_actor(actor):
            raise ValueError("not an actor, type is " + str(actor.type))

        return _get_or_insert_remote_user(actor, host)

    def send_follow_accept(self, follow_req: UserFollowRequest) -> None:
        inbox_url = follow_req.follower.inbox
        if inbox_url is None:
            raise ValueError("inbox url is not set")

        if follow_req.url is None:
            raise ValueError("follow request url is not set")

        res = self._session.post(
            inbox_url,
            json={
                "@context": [
                    "https://www.w3.org/ns/activitystreams",
                ],
                "type": "Accept",
                "object": follow_req.url,
            },
            timeout=self.default_timeout,
        )
        res.raise_for_status()

        with transaction.atomic():
            # create a new user follow
            actual_follow = UserFollow(
                follower=follow_req.follower,
                followee=follow_req.followee,
            )
            actual_follow.save()

            # delete the follow request
            follow_req.delete()


_req = Requester()


def get_requester() -> Requester:
    return _req


def _get_or_insert_remote_user(actor: Actor, hostname: str) -> User:
    # check if user already exists
    user = User.objects.filter(url=actor.id).first()
    if user is not None:
        return user

    # create a new remote user
    pprint(actor)
    new_user = User(
        username=actor.as_preferred_username,
        host=hostname,
        bpassword=None,
        nickname=actor.as_name,
        url=actor.id,
        inbox=actor.as_inbox.id,
        outbox=actor.as_outbox.id,
    )

    remote_info = RemoteUserInfo(
        user=new_user,
        following=actor.as_following.id if actor.as_following else None,
        followers=actor.as_followers.id if actor.as_followers else None,
        liked=actor.as_liked.id if actor.as_liked else None,
        preferred_username=actor.as_preferred_username,
    )

    # use transaction to ensure consistency
    with transaction.atomic():
        new_user.save()
        remote_info.save()

    return new_user
