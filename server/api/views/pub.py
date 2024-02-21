from rest_framework.views import APIView
from rest_framework import status
from rest_framework.response import Response
from api.parsers import ActivityJsonParser
from pprint import pprint
from api.serializers import pub
from api.requester import get_requester
from lightpub.settings import HOSTNAME, HTTP_SCHEME
import re
from api.models import User, UserFollowRequest
from django.shortcuts import get_object_or_404
from datetime import datetime


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

        obj = pub.Object.from_dict(data)
        if pub.is_follow(obj):
            follow = pub.FollowActivity.from_dict(data)
            process_follow_activity(follow)
            return Response(status=status.HTTP_202_ACCEPTED)

        return Response(status=status.HTTP_405_METHOD_NOT_ALLOWED)


class UserOutboxView(APIView):
    pass


class InvalidIDError(Exception):
    pass


scheme = HTTP_SCHEME
hostname = HOSTNAME
USER_PATTERN = rf"{scheme}://{hostname}/api/users/([a-f\d\-]+)/"


def _extract_local_user_id(uri: str) -> str:
    m = re.match(USER_PATTERN, uri)
    if m is None:
        raise InvalidIDError("Invalid user id")
    return m.group(1)


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
