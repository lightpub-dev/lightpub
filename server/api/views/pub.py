import logging
from datetime import datetime
from pprint import pprint

from django.db import transaction
from django.shortcuts import get_object_or_404
from rest_framework import status
from rest_framework.response import Response

from api import tasks
from api.models import Post, User, UserFollow, UserFollowRequest
from api.requester import get_requester
from api.serializers import pub
from api.utils.get_id import extract_local_post_id, extract_local_user_id

logger = logging.getLogger(__name__)


class InboxProcessingError(Exception):
    def __init__(self, status: int, response) -> None:
        self.status = status
        self.response = response


class UserInboxView:
    def post(self, request, user):
        # log headers
        # pprint(request.headers)
        # log request body
        print("request.data")
        pprint(request.data)
        # return method not allowed

        data = request.data[0]

        # TODO: check HTTP signature

        try:
            obj = pub.Object.from_dict(data)
            if pub.is_follow(obj):
                follow = pub.FollowActivity.from_dict(data)
                process_follow_activity(follow)
                return Response(status=status.HTTP_204_NO_CONTENT)
            elif pub.is_undo(obj):
                undo = pub.UndoActivity.from_dict(data)
                process_undo_activity(undo)
                return Response(status=status.HTTP_204_NO_CONTENT)
            elif pub.is_accept(obj):
                accept = pub.AcceptActivity.from_dict(data)
                process_accept_activity(accept)
                return Response(status=status.HTTP_204_NO_CONTENT)
            elif pub.is_reject(obj):
                reject = pub.RejectActivity.from_dict(data)
                process_reject_activity(reject)
                return Response(status=status.HTTP_204_NO_CONTENT)
            elif pub.is_create(obj):
                create = pub.CreateActivity.from_dict(data)
                process_create_activity(create)
                return Response(status=status.HTTP_204_NO_CONTENT)
            elif pub.is_announce(obj):
                announce = pub.AnnounceActivity.from_dict(data)
                process_announce_activity(announce)
                return Response(status=status.HTTP_204_NO_CONTENT)
            elif pub.is_delete(obj):
                delete = pub.DeleteActivity.from_dict(data)
                process_delete_activity(delete)
                return Response(status=status.HTTP_204_NO_CONTENT)
        except InboxProcessingError as e:
            return Response(status=e.status, data=e.response)

        return Response(status=status.HTTP_405_METHOD_NOT_ALLOWED)


class UserOutboxView:
    def post(self, request, user):
        pass


class InvalidIDError(Exception):
    pass


def _extract_local_user_id(uri: str) -> str:
    return extract_local_user_id(uri)


def _get_local_user_from_uri(uri: str) -> User:
    user_id = _extract_local_user_id(uri)
    return get_object_or_404(User, id=user_id)


def process_follow_activity(activity: pub.FollowActivity):
    target_user = _get_local_user_from_uri(activity.as_object.id)

    # fetch the actor's user id
    actor_id = activity.as_actor.id
    remote_user_result = tasks.fetch_remote_user.delay(id=actor_id)
    remote_user = User.objects.get(id=remote_user_result.get())

    # register follow request
    # check if already exists

    existing_fr = UserFollowRequest.objects.filter(
        uri=activity.id,
    ).first()
    if existing_fr:
        fr = existing_fr
        fr.incoming = True
        fr.created_at = datetime.now()
    else:
        fr = UserFollowRequest(
            uri=activity.id, follower=remote_user, followee=target_user, incoming=True
        )
        fr.save()

    tasks.send_follow_accept.delay(fr.id)


def process_reject_activity(activity: pub.RejectActivity):
    obj_s = activity.as_object
    if pub.is_follow(obj_s):
        obj = obj_s.reparse(pub.FollowActivity)
        actor_id = obj.as_actor.id
        object_id = obj.as_object.id

        # object should be a local user
        follower_internal_id = extract_local_user_id(actor_id)
        if follower_internal_id is None:
            raise InboxProcessingError(
                status=status.HTTP_400_BAD_REQUEST,
                response={"error": "invalid object id"},
            )

        follow = UserFollow.objects.filter(
            followee__uri=object_id,
            follower_id=follower_internal_id,
        ).first()
        if follow is None:
            raise InboxProcessingError(
                status=status.HTTP_404_NOT_FOUND,
                response={"error": "follow not found"},
            )

        follow.delete()
    else:
        raise InboxProcessingError(
            status.HTTP_406_NOT_ACCEPTABLE,
            {"error": "unsupported accept activity type"},
        )


def process_accept_activity(activity: pub.AcceptActivity):
    obj_s = activity.as_object
    if pub.is_follow(obj_s):
        obj = obj_s.reparse(pub.FollowActivity)
        actor_id = obj.as_actor.id
        object_id = obj.as_object.id

        # actor should be a local user
        follower_internal_id = extract_local_user_id(actor_id)
        if follower_internal_id is None:
            raise InboxProcessingError(
                status=status.HTTP_400_BAD_REQUEST,
                response={"error": "invalid object id"},
            )

        follow_req = UserFollowRequest.objects.filter(
            followee__uri=object_id,
            follower_id=follower_internal_id,
        ).first()
        if follow_req is None:
            raise InboxProcessingError(
                status=status.HTTP_404_NOT_FOUND,
                response={"error": "follow request not found"},
            )
        if follow_req.incoming:
            raise InboxProcessingError(
                status=status.HTTP_403_FORBIDDEN,
                response={"error": "you cannot accept this follow request"},
            )

        with transaction.atomic():
            # create a new user follow
            UserFollow.objects.update_or_create(
                follower=follow_req.follower,
                followee=follow_req.followee,
                defaults={"created_at": datetime.now()},
            )
            follow_req.delete()
    else:
        raise InboxProcessingError(
            status.HTTP_406_NOT_ACCEPTABLE,
            {"error": "unsupported accept activity type"},
        )


def process_undo_activity(activity: pub.UndoActivity):
    obj_s = activity.as_object
    if pub.is_follow(obj_s):
        obj = obj_s.reparse(pub.FollowActivity)
        actor = obj.as_actor
        follower_uri = obj.as_actor.id
        followee_uri = obj.as_object.id

        if actor.id != follower_uri:
            raise InboxProcessingError(
                status=status.HTTP_403_FORBIDDEN,
                response={"error": "actor id does not match follower id"},
            )

        followee_id = extract_local_user_id(followee_uri)
        if followee_id is None:
            raise InboxProcessingError(
                status=status.HTTP_400_BAD_REQUEST,
                response={"error": "invalid followee id"},
            )

        # get User object of follower to get the id
        follower = User.objects.filter(uri=follower_uri).first()
        if follower is None:
            raise InboxProcessingError(
                status=status.HTTP_400_BAD_REQUEST,
                response={"error": "follower not found"},
            )
        follower_id = follower.id

        UserFollow.objects.filter(
            follower_id=follower_id, followee_id=followee_id
        ).delete()

        return

    raise InboxProcessingError(
        status.HTTP_400_BAD_REQUEST,
        {"error": "unsupported undo activity type"},
    )


def process_create_activity(activity: pub.CreateActivity):
    obj = activity.as_object
    if pub.is_note(obj):
        note = obj.reparse(pub.Note)
        # pprint(note)

        user_result = tasks.fetch_remote_user(id=activity.as_actor.id)
        user = User.objects.get(id=user_result.get())

        # fetch reply to
        reply_to = note.as_in_reply_to
        logger.info("reply_to: %s", reply_to)
        if reply_to:
            ref_post = _get_or_insert_post_from_uri(reply_to.id)
        else:
            ref_post = None

        post = Post(
            uri=note.id,
            poster=user,
            content=note.as_content,
            created_at=note.as_published.as_datetime(),
            privacy=0,  # TODO: implement privacy
            reply_to=ref_post,  # TODO: implement reply_to
        )
        post.save()

        return

    raise InboxProcessingError(
        status.HTTP_400_BAD_REQUEST,
        {"error": "unsupported create activity type"},
    )


def process_announce_activity(activity: pub.AnnounceActivity):
    obj = activity.as_object
    # assume obj is a Note

    user_result = tasks.fetch_remote_user(id=activity.as_actor.id)
    user = User.objects.get(id=user_result.get())

    ref_post = _get_or_insert_post_from_uri(obj.id)

    post = Post(
        uri=obj.id,
        poster=user,
        content=None,
        created_at=activity.as_published.as_datetime(),
        privacy=0,  # TODO: implement privacy
        reply_to=None,
        repost_of=ref_post,
    )
    post.save()

    return


def process_delete_activity(activity: pub.DeleteActivity):
    obj = activity.as_object
    # assume obj is a Note

    obj_id = obj.id
    local_post_id = extract_local_post_id(obj_id)
    if local_post_id:
        post = Post.objects.filter(id=local_post_id).first()
    else:
        post = Post.objects.filter(uri=obj_id).first()

    if post is None:
        raise InboxProcessingError(
            status=status.HTTP_404_NOT_FOUND,
            response={"error": "post not found"},
        )
    post.deleted_at = activity.as_published.as_datetime()
    post.save()


def _get_or_insert_post_from_uri(uri: str) -> Post:
    local_post_id = extract_local_post_id(uri)
    if local_post_id:
        # TODO: visibility check
        ref_post = Post.objects.filter(id=local_post_id).first()
        if ref_post is None:
            logger.debug(
                "referenced post not found: %s (local id: %s)", uri, local_post_id
            )
            raise InboxProcessingError(
                status=status.HTTP_404_NOT_FOUND,
                response={"error": "referenced post not found"},
            )
    else:
        ref_post_result = tasks.fetch_remote_post_by_uri.delay(uri)
        ref_post = Post.objects.get(id=ref_post_result.get())

    return ref_post
