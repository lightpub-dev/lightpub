from ..serializers.post import PostSerializer, UploadedFileSerializer
from ..serializers.user import SimpleUserSerializer
from ..models import Post, UploadedFile, PostAttachment, User, PostFavorite
from rest_framework import status, mixins, views, generics
from rest_framework.response import Response
from ..auth import AuthOnlyPermission, NoAuthPermission, NoPermission
from rest_framework.viewsets import ModelViewSet, GenericViewSet
from django.db.models import Q
from .users import UserSpecifier
from django.shortcuts import get_object_or_404
from django.http import HttpResponse
from PIL import Image
from django.views.decorators.cache import cache_control
from django.utils.decorators import method_decorator


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


class PostInteractionViewBase(generics.ListAPIView):
    permission_classes = [NoAuthPermission]
    serializer_class = PostSerializer

    def get_queryset(self):
        user = self.request.user
        if not user.id:
            user = None

        post_id = self.kwargs["pk"]
        target_post = get_object_or_404(Post, id=post_id)
        self.check_object_permissions(self.request, target_post)
        return self._fetch(user, target_post)

    def _fetch(self, user: User | None, target_post: Post):
        raise NotImplementedError()


class ReplyListView(PostInteractionViewBase):
    def _fetch(self, user: User | None, target_post: Post):
        if user:
            return (
                target_post.replies.distinct()
                .filter(
                    Q(privacy__in=[0, 1])
                    | Q(privacy=2, poster__followers__follower=user)
                )
                .order_by("-created_at")
            )

        return (
            target_post.replies.distinct()
            .filter(privacy__in=[0, 1])
            .order_by("-created_at")
        )


class QuoteListView(PostInteractionViewBase):
    def _fetch(self, user: User | None, target_post: Post):
        if user:
            return (
                target_post.reposts.distinct()
                .filter(content__isnull=False)
                .filter(
                    Q(privacy__in=[0, 1])
                    | Q(privacy=2, poster__followers__follower=user)
                )
                .order_by("-created_at")
            )

        return (
            target_post.reposts.distinct()
            .filter(content__isnull=False)
            .filter(privacy__in=[0, 1])
            .order_by("-created_at")
        )


class RepostListView(generics.ListAPIView):
    permission_classes = [NoAuthPermission]
    serializer_class = SimpleUserSerializer

    def get_queryset(self):
        user = self.request.user
        if not user.id:
            user = None

        post_id = self.kwargs["pk"]
        target_post = get_object_or_404(Post, id=post_id)
        self.check_object_permissions(self.request, target_post)

        if user:
            repost_users = (
                target_post.reposts.distinct()
                .filter(content__isnull=True)
                .filter(
                    Q(privacy__in=[0, 1])
                    | Q(privacy=2, poster__followers__follower=user)
                )
                .order_by("-created_at")
                .values_list("poster", flat=True)
                .distinct()
            )
        else:
            repost_users = (
                target_post.reposts.distinct()
                .filter(content__isnull=True)
                .filter(privacy__in=[0, 1])
                .order_by("-created_at")
                .values_list("poster", flat=True)
                .distinct()
            )

        # extract user
        return repost_users


class FavoriteListView(generics.ListAPIView):
    permission_classes = [NoAuthPermission]
    serializer_class = SimpleUserSerializer

    def get_queryset(self):
        user = self.request.user
        if not user.id:
            user = None

        post_id = self.kwargs["pk"]
        target_post = get_object_or_404(Post, id=post_id)
        self.check_object_permissions(self.request, target_post)

        favorite_users = (
            PostFavorite.objects.filter(post=target_post)
            .order_by("-created_at")
            .values_list("user", flat=True)
            .distinct()
        )

        # extract user
        return favorite_users
