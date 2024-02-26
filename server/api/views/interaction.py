from django.shortcuts import get_object_or_404
from rest_framework import mixins, status, viewsets
from rest_framework.response import Response

from ..auth import AuthOnlyPermission
from ..models import PostBookmark, PostFavorite, PostReaction
from ..serializers.interaction import (
    PostBookmarkSerializer,
    PostFavoriteSerializer,
    PostReactionSerializer,
)


class PostFavoriteView(
    mixins.CreateModelMixin,
    mixins.DestroyModelMixin,
    mixins.ListModelMixin,
    mixins.RetrieveModelMixin,
    viewsets.GenericViewSet,
):
    serializer_class = PostFavoriteSerializer
    permission_classes = [AuthOnlyPermission]

    def get_queryset(self):
        favorites = PostFavorite.objects.filter(user=self.request.user).order_by(
            "-created_at"
        )
        return favorites

    def get_object(self):
        # get path paramter pk
        pk = self.kwargs["pk"]
        # treat pk as post_id
        return get_object_or_404(PostFavorite, user=self.request.user, post=pk)

    def get_serializer_context(self):
        return {"request": self.request}


class PostBookmarkView(
    mixins.CreateModelMixin,
    mixins.DestroyModelMixin,
    mixins.ListModelMixin,
    mixins.RetrieveModelMixin,
    viewsets.GenericViewSet,
):
    serializer_class = PostBookmarkSerializer
    permission_classes = [AuthOnlyPermission]

    def get_queryset(self):
        favorites = PostBookmark.objects.filter(user=self.request.user).order_by(
            "-created_at"
        )
        return favorites

    def get_object(self):
        # get path paramter pk
        pk = self.kwargs["pk"]
        # treat pk as post_id
        return get_object_or_404(PostBookmark, user=self.request.user, post=pk)

    def get_serializer_context(self):
        return {"request": self.request}


class PostReactionView(
    mixins.CreateModelMixin,
    mixins.DestroyModelMixin,
    viewsets.GenericViewSet,
):
    serializer_class = PostReactionSerializer
    permission_classes = [AuthOnlyPermission]

    def get_serializer_context(self):
        return {
            "request": self.request,
        }

    def get_object(self):
        pk = self.kwargs["pk"]
        user = self.request.user
        # get emoji from query parameter
        emoji = self.request.query_params.get("emoji", None)
        if emoji is None:
            return Response({"emoji": "not set"}, status=status.HTTP_400_BAD_REQUEST)

        return get_object_or_404(PostReaction, user=user, post=pk, emoji=emoji)

    def get_queryset(self):
        user = self.request.user

        return PostReaction.objects.filter(user=user).order_by("-created_at")
