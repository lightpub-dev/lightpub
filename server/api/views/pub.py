from datetime import datetime
from pprint import pprint

from django.shortcuts import get_object_or_404
from rest_framework import status
from rest_framework.response import Response
from rest_framework.views import APIView

from api.models import User, UserFollow, UserFollowRequest
from api.parsers import ActivityJsonParser
from api.requester import get_requester
from api.serializers import pub
from api.serializers.user import extract_local_user_id


class InboxProcessingError(Exception):
    def __init__(self, status: int, response) -> None:
        self.status = status
        self.response = response


class UserInboxView(APIView):
    parser_classes = [ActivityJsonParser]

    def post(self, request, user_spec):
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
                return Response(status=status.HTTP_202_ACCEPTED)
            elif pub.is_undo(obj):
                undo = pub.UndoActivity.from_dict(data)
                process_undo_activity(undo)
                return Response(status=status.HTTP_202_ACCEPTED)
        except InboxProcessingError as e:
            return Response(status=e.status, data=e.response)

        return Response(status=status.HTTP_405_METHOD_NOT_ALLOWED)


class UserOutboxView(APIView):
    pass


class InvalidIDError(Exception):
    pass


def _extract_local_user_id(uri: str) -> str:
    return extract_local_user_id(uri)


def _get_local_user_from_uri(uri: str) -> User:
    user_id = _extract_local_user_id(uri)
    return get_object_or_404(User, id=user_id)


def process_follow_activity(activity: pub.FollowActivity):
    req = get_requester()

    target_user = _get_local_user_from_uri(activity.as_object.id)

    # fetch the actor's user id
    actor_id = activity.as_actor.id
    remote_user = req.fetch_remote_user(id=actor_id)

    # register follow request
    # check if already exists

    existing_fr = UserFollowRequest.objects.filter(
        url=activity.id,
    ).first()
    if existing_fr:
        fr = existing_fr
        fr.incoming = True
        fr.created_at = datetime.now()
    else:
        fr = UserFollowRequest(
            url=activity.id, follower=remote_user, followee=target_user, incoming=True
        )
        fr.save()

    req.send_follow_accept(fr)


def process_undo_activity(activity: pub.UndoActivity):
    # req = get_requester()

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
        follower = User.objects.filter(url=follower_uri).first()
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
