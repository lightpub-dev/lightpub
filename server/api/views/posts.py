from django.db.models import Q
from django.http import HttpResponse
from django.shortcuts import get_object_or_404
from django.utils.decorators import method_decorator
from django.views.decorators.cache import cache_control
from PIL import Image
from rest_framework import mixins, status, views
from rest_framework.decorators import action
from rest_framework.response import Response
from rest_framework.viewsets import GenericViewSet, ModelViewSet

from api.serializers.interaction import PostBookmarkSerializer, PostFavoriteSerializer

from ..auth import AuthOnlyPermission, NoAuthPermission, NoPermission
from ..models import Post, PostAttachment, PostBookmark, PostFavorite, UploadedFile
from ..serializers.post import (
    PostAddToListSerializer,
    PostSerializer,
    UploadedFileSerializer,
)
from .users import UserSpecifier


class PostViewSet(ModelViewSet):
    serializer_class = PostSerializer

    def get_serializer_context(self):
        return {
            "request": self.request,
        }

    def list(self, request, *args, **kwargs):
        # filter privacy==1 posts
        queryset = self.filter_queryset(self.get_queryset())
        queryset = queryset.exclude(privacy=1)
        page = self.paginate_queryset(queryset)
        if page is not None:
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
        elif self.action in ["favorites", "bookmarks"]:
            if self.request.method == "GET":
                permission_classes = [NoAuthPermission]
            else:
                permission_classes = [AuthOnlyPermission]
        else:
            permission_classes = [NoAuthPermission]
        return [permission() for permission in permission_classes]

    def get_queryset(self):
        user = self.request.user
        authed = bool(user.id)  # type: ignore

        if not authed:
            return Post.objects.filter(privacy__in=[0, 1]).order_by("-created_at")

        posts = (
            Post.objects.distinct()
            .filter(
                Q(privacy__in=[0, 1])
                | Q(privacy=2, poster__followers__follower=user)
                | Q(privacy=3, poster=user)
            )
            .filter(
                deleted_at__isnull=True,
            )
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

    @action(
        detail=True,
        methods=["GET", "PUT", "DELETE"],
        url_path="favorites",
        url_name="favorites",
        serializer_class=PostAddToListSerializer,
    )
    def favorites(self, request, pk=None):
        post = self.get_object()
        if self.request.method == "GET":
            favorites = post.favorites.all()
            page = self.paginate_queryset(favorites)
            if page is not None:
                serializer = PostFavoriteSerializer(
                    page,
                    many=True,
                    context={
                        "request": request,
                    },
                )
                return self.get_paginated_response(serializer.data)

            serializer = PostFavoriteSerializer(
                favorites,
                many=True,
                context={
                    "request": request,
                },
            )
            return Response(serializer.data)
        else:
            user = request.user
            if self.request.method == "PUT":
                PostFavorite.objects.create(user=user, post=post)
                return Response(status=status.HTTP_201_CREATED)
            elif self.request.method == "DELETE":
                PostFavorite.objects.filter(user=user, post=post).delete()
                return Response(status=status.HTTP_204_NO_CONTENT)
            else:
                raise ValueError("invalid method")

    @action(
        detail=True,
        methods=["GET", "PUT", "DELETE"],
        url_path="bookmarks",
        url_name="bookmarks",
        serializer_class=PostAddToListSerializer,
    )
    def bookmarks(self, request, pk=None):
        post = self.get_object()
        if self.request.method == "GET":
            bookmarks = post.bookmarks.all()
            page = self.paginate_queryset(bookmarks)
            if page is not None:
                serializer = PostBookmarkSerializer(
                    page,
                    many=True,
                    context={
                        "request": request,
                    },
                )
                return self.get_paginated_response(serializer.data)

            serializer = PostBookmarkSerializer(
                bookmarks,
                many=True,
                context={
                    "request": request,
                },
            )
            return Response(serializer.data)
        else:
            user = request.user
            if self.request.method == "PUT":
                PostBookmark.objects.create(user=user, post=post)
                return Response(status=status.HTTP_201_CREATED)
            elif self.request.method == "DELETE":
                PostBookmark.objects.filter(user=user, post=post).delete()
                return Response(status=status.HTTP_204_NO_CONTENT)
            else:
                raise ValueError("invalid method")

    def _return_posts(self, request, posts, serializer_context: dict = {}):
        page = self.paginate_queryset(posts)
        if page is not None:
            serializer = PostSerializer(
                page,
                many=True,
                context={
                    "request": request,
                }
                | serializer_context,
            )
            return self.get_paginated_response(serializer.data)

        serializer = PostSerializer(
            posts,
            many=True,
            context={
                "request": request,
            }
            | serializer_context,
        )
        return Response(serializer.data)

    @action(detail=True, methods=["GET"], url_path="replies", url_name="replies")
    def replies(self, request, pk=None):
        post = self.get_object()
        user = request.user
        if user.id:
            posts = (
                post.replies.distinct()
                .filter(
                    Q(privacy__in=[0, 1])
                    | Q(privacy=2, poster__followers__follower=user)
                )
                .order_by("-created_at")
            )
        else:
            posts = (
                post.replies.distinct()
                .filter(privacy__in=[0, 1])
                .order_by("-created_at")
            )

        return self._return_posts(request, posts)

    @action(detail=True, methods=["GET"], url_path="quotes", url_name="quotes")
    def quotes(self, request, pk=None):
        post = self.get_object()
        user = request.user
        if user.id:
            posts = (
                post.reposts.distinct()
                .filter(content__isnull=False)
                .filter(
                    Q(privacy__in=[0, 1])
                    | Q(privacy=2, poster__followers__follower=user)
                )
                .order_by("-created_at")
            )
        else:
            posts = (
                post.reposts.distinct()
                .filter(content__isnull=False)
                .filter(privacy__in=[0, 1])
                .order_by("-created_at")
            )

        return self._return_posts(request, posts)

    @action(detail=True, methods=["GET"], url_path="reposts", url_name="reposts")
    def reposts(self, request, pk=None):
        post = self.get_object()
        user = request.user
        if user.id:
            reposts = (
                post.reposts.distinct()
                .filter(content__isnull=True)
                .filter(
                    Q(privacy__in=[0, 1])
                    | Q(privacy=2, poster__followers__follower=user)
                )
                .order_by("-created_at")
            )
        else:
            reposts = (
                post.reposts.distinct()
                .filter(content__isnull=True)
                .filter(privacy__in=[0, 1])
                .order_by("-created_at")
            )

        return self._return_posts(
            request,
            reposts,
            serializer_context={
                "nested_repost": 0,
            },
        )


class UploadFileView(
    mixins.CreateModelMixin,
    mixins.DestroyModelMixin,
    mixins.RetrieveModelMixin,
    GenericViewSet,
):
    serializer_class = UploadedFileSerializer
    queryset = UploadedFile.objects.all()
    permission_classes = [AuthOnlyPermission]

    def get_serializer_context(self):
        return {
            "request": self.request,
        }


class PostAttachmentView(views.APIView):
    permission_classes = [NoAuthPermission]

    @method_decorator(cache_control(max_age=60 * 60 * 24 * 7))
    def get(self, request, pk):
        attachment = get_object_or_404(PostAttachment, id=pk)

        # permission check
        self.check_object_permissions(request, attachment)

        file = attachment.file
        if not file:
            return Response(status=status.HTTP_410_GONE)

        file = file.file
        # serve actual file
        # file is an image file
        # use Pillow to decide content_type
        image = Image.open(file)
        content_type = image.format.lower()
        content_type = f"image/{content_type}"
        return HttpResponse(file, content_type=content_type)
