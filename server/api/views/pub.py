from rest_framework.views import APIView
from rest_framework import status
from rest_framework.response import Response
from api.parsers import ActivityJsonParser
from pprint import pprint
from api.serializers.user import create_new_user

from api.requester import get_requester


class UserInboxView(APIView):
    parser_classes = [ActivityJsonParser]

    def post(self, request, user_spec):
        # log headers
        pprint(request.headers)
        # log request body
        pprint(request.data)
        # return method not allowed
        return Response(status=status.HTTP_405_METHOD_NOT_ALLOWED)


class UserOutboxView(APIView):
    pass


def process_follow_activity(activity):
    req = get_requester()

    # fetch the actor's user id
    actor_id = activity["actor"]["@id"]
    actor = req.fetch_remote_user(id=actor_id)
    # create a new user
    create_new_user("misskey", "testtest", actor["name"], "misskey.tinax.local")

    req.send_follow_accept()
