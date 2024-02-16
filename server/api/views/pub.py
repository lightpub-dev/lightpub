from rest_framework.views import APIView
from rest_framework import status
from rest_framework.response import Response
from api.parsers import ActivityJsonParser
from pprint import pprint


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
