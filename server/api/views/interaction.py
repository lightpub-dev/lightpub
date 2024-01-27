from rest_framework import viewsets, mixins, views, status
from rest_framework.response import Response
from ..serializers.interaction import PostFavoriteSerializer, PostBookmarkSerializer
from ..auth import AuthOnlyPermission
from ..models import PostFavorite, PostBookmark
from django.shortcuts import get_object_or_404


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
