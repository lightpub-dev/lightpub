from django.http import Http404
from rest_framework import generics
from rest_framework.exceptions import APIException
from rest_framework.response import Response

from api import tasks
from api.auth.permission import AuthOnlyPermission
from api.models import UserFollow
from api.serializers.follow import CreateFollowSerializer, FollowSerializer


class SameUserException(APIException):
    status_code = 400
    default_detail = "You cannot follow yourself"
    default_code = "same_user"


class CreateFollowView(generics.CreateAPIView):
    serializer_class = CreateFollowSerializer
    permission_classes = [AuthOnlyPermission]

    def create(self, request, *args, **kwargs):
        ser = self.get_serializer(data=request.data)
        ser.is_valid(raise_exception=True)

        ser.save()

        return Response(status=201)


class FollowView(generics.RetrieveDestroyAPIView):
    serializer_class = FollowSerializer
    permission_classes = [AuthOnlyPermission]

    def get_object(self):
        my = self.request.user
        follow_target = self.kwargs["user"].get_user_model()
        if follow_target is None:
            raise Http404("user not found")
        if my.id == follow_target.id:
            raise SameUserException()
        uf = UserFollow.objects.filter(follower=my, followee=follow_target).first()
        if uf is None:
            raise Http404("follow not found")
        return uf

    def destroy(self, request, *args, **kwargs):
        object = self.get_object()

        tasks.send_unfollow.delay(self.request.user.id, object.followee.id)

        return Response(status=204)
