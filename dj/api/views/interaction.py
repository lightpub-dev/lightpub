from rest_framework import viewsets, mixins, views, status
from rest_framework.response import Response
from ..serializers.interaction import PostFavoriteSerializer, PostBookmarkSerializer
from ..auth import AuthOnlyPermission
from ..models import PostFavorite, PostBookmark


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

    def get_serializer_context(self):
        return {"request": self.request}


class PostFavoriteByPostId(views.APIView):
    permission_classes = [AuthOnlyPermission]

    def delete(self, request, post_id):
        favorite = PostFavorite.objects.filter(user=request.user, post_id=post_id)
        favorite.delete()
        return Response(status=status.HTTP_204_NO_CONTENT)


class PostBookmarkByPostId(views.APIView):
    permission_classes = [AuthOnlyPermission]

    def delete(self, request, post_id):
        favorite = PostBookmark.objects.filter(user=request.user, post_id=post_id)
        favorite.delete()
        return Response(status=status.HTTP_204_NO_CONTENT)
