from rest_framework import viewsets, mixins
from ..serializers.interaction import PostFavoriteSerializer, PostBookmarkSerializer
from ..auth import AuthOnlyPermission
from ..models import PostFavorite


class PostFavoriteView(
    mixins.CreateModelMixin,
    mixins.DestroyModelMixin,
    mixins.ListModelMixin,
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
