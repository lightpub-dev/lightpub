from ..serializers.post import PostSerializer
from ..models import Post, UserFollow
from rest_framework import generics, status
from rest_framework.response import Response
from ..auth import TokenAuth, AuthOnlyPermission, NoAuthPermission, NoPermission
from rest_framework.viewsets import ModelViewSet
from django.db.models import Q


class PostViewSet(ModelViewSet):
    serializer_class = PostSerializer

    def destroy(self, request, pk):
        post = Post.objects.get(pk=pk)
        if post.poster == request.user:
            return super().destroy(request, pk)

        return Response(
            {"detail": "You can delete only your own posts"},
            status=status.HTTP_403_FORBIDDEN,
        )

    def get_permissions(self):
        if self.action == "create":
            permission_classes = [AuthOnlyPermission]
        elif self.action in ["update", "partial_update"]:
            permission_classes = [NoPermission]
        else:
            permission_classes = [NoAuthPermission]
        return [permission() for permission in permission_classes]

    def get_queryset(self):
        user = self.request.user
        authed = not not user.id  # type: ignore

        if not authed:
            return Post.objects.filter(privacy__in=[0, 1]).order_by("-created_at")

        return Post.objects.filter(
            Q(privacy__in=[0, 1]) | Q(privacy=2, poster__followers__follower=user)
        ).order_by("-created_at")
