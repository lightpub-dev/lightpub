from rest_framework import generics, mixins
from rest_framework.response import Response
from rest_framework.reverse import reverse

from api.auth.permission import AuthOnlyPermission
from api.serializers.follow import CreateFollowSerializer
from api.utils.users import UserSpecifier

from ..jsonld.mixins import JsonldAwareMixin
from ..models import UserFollow
from ..serializers.user import UserFollowSerializer


class CreateFollowView(generics.CreateAPIView):
    serializer_class = CreateFollowSerializer
    permission_classes = [AuthOnlyPermission]

    def create(self, request, *args, **kwargs):
        ser = self.get_serializer(data=request.data)
        ser.is_valid(raise_exception=True)

        ser.save()

        return Response(status=201)


class UserFollowerView(
    JsonldAwareMixin, mixins.ListModelMixin, generics.GenericAPIView
):
    serializer_class = UserFollowSerializer
    permission_classes = []

    def list(self, request, *args, **kwargs):
        if not self.jsonld_requested():
            return super().list(request, *args, **kwargs)

        queryset = self.filter_queryset(self.get_queryset())
        page = self.paginate_queryset(queryset)
        if page is not None:
            followers = [follower.follower for follower in page]
            urls = [
                reverse("api:user-detail", kwargs={"pk": follower.id}, request=request)
                for follower in followers
            ]
            return self.get_paginated_response(urls)

        raise ValueError("page is None")

    def get_queryset(self):
        target_user = UserSpecifier.parse_str(self.kwargs["user_spec"]).get_user_model()
        return UserFollow.objects.filter(followee=target_user).order_by("-created_at")

    def get_serializer_context(self):
        return {
            "request": self.request,
        }


class UserFollowingView(
    JsonldAwareMixin,
    mixins.ListModelMixin,
    generics.GenericAPIView,
):
    serializer_class = UserFollowSerializer
    permission_classes = []

    def list(self, request, *args, **kwargs):
        if not self.jsonld_requested():
            return super().list(request, *args, **kwargs)

        queryset = self.filter_queryset(self.get_queryset())

        page = self.paginate_queryset(queryset)
        if page is not None:
            followees = [followee.followee for followee in page]
            urls = [
                reverse("api:user-detail", kwargs={"pk": followee.id}, request=request)
                for followee in followees
            ]
            return self.get_paginated_response(urls)

        raise ValueError("page is None")

    def get_queryset(self):
        target_user = UserSpecifier.parse_str(self.kwargs["user_spec"]).get_user_model()
        return UserFollow.objects.filter(follower=target_user).order_by("-created_at")

    def get_serializer_context(self):
        return {
            "request": self.request,
        }
