from django.http import Http404, HttpResponse
from django.utils.decorators import method_decorator
from django.views.decorators.cache import cache_control
from PIL import Image
from rest_framework import generics, mixins, status, views, viewsets
from rest_framework.decorators import action
from rest_framework.response import Response
from rest_framework.reverse import reverse

from api.auth.permission import OwnerOnlyPermission
from api.jsonld.mixins import JsonldMixin
from api.serializers.interaction import PostBookmarkSerializer, PostFavoriteSerializer
from api.utils.users import UserSpecifier

from ..auth import NoAuthPermission
from ..models import User
from ..serializers.user import (
    DetailedUserSerializer,
    JsonldDetailedUserSerializer,
    LoginSerializer,
    RegisterSerializer,
    UserFollowerSerializer,
    login_and_generate_token,
)
from .pub import UserInboxView, UserOutboxView


# Create your views here.
class RegisterView(generics.CreateAPIView):
    permission_classes = [NoAuthPermission]
    serializer_class = RegisterSerializer
    queryset = User.objects.all()


class LoginView(views.APIView):
    permission_classes = []
    authorization_classes = []
    serializer_class = LoginSerializer

    def post(self, request):
        ser = LoginSerializer(data=request.data)
        ser.is_valid(raise_exception=True)
        username = ser.validated_data["username"]
        password = ser.validated_data["password"]
        token = login_and_generate_token(username, password)
        if token is None:
            return Response(
                {"error": "invalid username or password"},
                status=status.HTTP_401_UNAUTHORIZED,
            )
        return Response({"token": token})


class UserViewset(
    JsonldMixin,
    mixins.RetrieveModelMixin,
    mixins.ListModelMixin,
    mixins.UpdateModelMixin,
    viewsets.GenericViewSet,
):
    permission_classes = [NoAuthPermission]
    queryset = User.objects.all()
    normal_serializer_class = DetailedUserSerializer
    jsonld_serializer_class = JsonldDetailedUserSerializer

    def get_serializer_context(self):
        return {
            "request": self.request,
        }

    def get_object(self):
        pk = self.kwargs["pk"]
        user_spec = UserSpecifier.parse_str(pk)
        user = user_spec.get_user_model()
        if user is None:
            raise Http404("user not found")
        return user

    @action(
        detail=True, methods=["GET"], url_path="followers", url_name="followers-list"
    )
    def list_followers(self, request, pk=None):
        user = self.get_object()
        followers = user.followers.order_by("-created_at").all()

        page = self.paginate_queryset(followers)
        if page is None:
            raise ValueError("page is None")

        if not self.jsonld_requested():
            data = UserFollowerSerializer(page, many=True).data
        else:
            actual_followers = [follow.follower for follow in page]
            urls = [
                reverse(
                    "api:user-detail",
                    kwargs={"pk": follower.id},
                    request=request,
                )
                for follower in actual_followers
            ]
            data = urls

        return self.get_paginated_response(data)

    @action(
        detail=True, methods=["GET"], url_path="following", url_name="following-list"
    )
    def list_following(self, request, pk=None):
        user = self.get_object()
        followees = user.followings.order_by("-created_at").all()

        page = self.paginate_queryset(followees)
        if page is None:
            raise ValueError("page is None")

        if not self.jsonld_requested():
            data = UserFollowerSerializer(page, many=True).data
        else:
            actual_followees = [follow.followee for follow in page]
            urls = [
                reverse(
                    "api:user-detail",
                    kwargs={"pk": followee.id},
                    request=request,
                )
                for followee in actual_followees
            ]
            data = urls

        return self.get_paginated_response(data)

    @action(
        detail=True,
        methods=["GET"],
        url_path="favorites",
        url_name="favorites",
    )
    def favorites(self, request, pk=None):
        user = self.get_object()
        favorites = user.favorites.order_by("-created_at").all()

        page = self.paginate_queryset(favorites)
        if page is None:
            raise ValueError("page is None")

        if not self.jsonld_requested():
            data = PostFavoriteSerializer(page, many=True).data
        else:
            post_urls = []
            for favorite in page:
                post_urls.append(
                    reverse(
                        "api:post-detail",
                        kwargs={"pk": favorite.post.id},
                        request=request,
                    )
                )
            data = post_urls

        return self.get_paginated_response(data)

    @action(
        detail=True,
        methods=["GET"],
        url_path="bookmarks",
        url_name="bookmarks",
        permission_classes=[OwnerOnlyPermission],
    )
    def bookmarks(self, request, pk=None):
        user = self.get_object()
        bookmarks = user.bookmarks.order_by("-created_at").all()

        page = self.paginate_queryset(bookmarks)
        if page is None:
            raise ValueError("page is None")

        # bookmarks are not accessed from external servers,
        # so we don't have to consider jsonld reponses.
        data = PostBookmarkSerializer(page, many=True).data
        return self.get_paginated_response(data)

    @action(detail=True, methods=["POST"], url_path="inbox", url_name="inbox")
    def inbox(self, request, pk=None):
        user = self.get_object()
        inbox = UserInboxView()
        return inbox.post(request, user)

    @action(detail=True, methods=["POST"], url_path="outbox", url_name="outbox")
    def outbox(self, request, pk=None):
        user = self.get_object()
        outbox = UserOutboxView()
        return outbox.post(request, user)


class UserAvatarView(views.APIView):
    permission_classes = [NoAuthPermission]

    @method_decorator(cache_control(max_age=60 * 60 * 24))
    def get(self, request, user_spec):
        user_spec = UserSpecifier.parse_str(user_spec)
        user = user_spec.get_user_model()
        if not user:
            raise Http404("user not found")
        if user.avatar:
            file = user.avatar.file
            image = Image.open(file)
            content_type = image.format.lower()
            content_type = f"image/{content_type}"
            return HttpResponse(file, content_type=content_type)
        else:
            raise Http404("avatar not set")
