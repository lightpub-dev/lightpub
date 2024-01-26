from ..serializers.post import PostSerializer
from ..models import Post, UserFollow
from rest_framework import generics, status
from rest_framework.response import Response
from ..auth import TokenAuth, AuthOnlyPermission, NoAuthPermission, NoPermission
from rest_framework.viewsets import ModelViewSet
from django.db.models import Q
from .users import UserSpecifier
from api.pagination import MyPagination


class PostViewSet(ModelViewSet):
    serializer_class = PostSerializer

    def list(self, request, *args, **kwargs):
        # filter privacy==1 posts
        queryset = self.filter_queryset(self.get_queryset())
        queryset = queryset.exclude(privacy=1)
        page = self.paginate_queryset(queryset)
        if page:
            serializer = self.get_serializer(page, many=True)
            return self.get_paginated_response(serializer.data)

        serializer = self.get_serializer(queryset, many=True)
        return Response(serializer.data)

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
        authed = bool(user.id)  # type: ignore

        if not authed:
            return Post.objects.filter(privacy__in=[0, 1]).order_by("-created_at")

        posts = Post.objects.distinct().filter(
            Q(privacy__in=[0, 1])
            | Q(privacy=2, poster__followers__follower=user)
            | Q(privacy=3, poster=user)
        )

        hashtag = self.request.query_params.get("hashtag", None)
        if hashtag:
            posts = posts.filter(hashtags__hashtag=hashtag)

        user = self.request.query_params.get("user", None)
        if user:
            user_spec = UserSpecifier.parse_str(user)
            if user_spec.user_id:
                posts = posts.filter(poster__id=user_spec.user_id)
            elif user_spec.username_and_host:
                posts = posts.filter(
                    poster__username=user_spec.username_and_host[0],
                    poster__host=user_spec.username_and_host[1],
                )
            else:
                raise ValueError("user_id and username_and_host cannot both be None")

        posts = posts.order_by("-created_at")

        return posts
