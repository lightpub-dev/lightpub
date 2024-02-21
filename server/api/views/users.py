from django.http import Http404, HttpResponse
from django.shortcuts import get_object_or_404
from rest_framework import generics, mixins, status, views, viewsets
from rest_framework.response import Response
from api.jsonld.mixins import JsonldAwareMixin, JsonldMixin

from api.utils.users import UserSpecifier
from ..auth import AuthOnlyPermission, NoAuthPermission
from ..models import User, UserFollow
from ..serializers.user import (
    DetailedUserSerializer,
    JsonldDetailedUserSerializer,
    LoginSerializer,
    RegisterSerializer,
    login_and_generate_token,
    UserFollowSerializer,
    UserFollowerSerializer,
)
from PIL import Image
from django.views.decorators.cache import cache_control
from django.utils.decorators import method_decorator
from rest_framework.reverse import reverse


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


class UserFollowingViewset(
    JsonldAwareMixin,
    mixins.CreateModelMixin,
    mixins.DestroyModelMixin,
    mixins.ListModelMixin,
    viewsets.GenericViewSet,
):
    serializer_class = UserFollowSerializer

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

    def get_permissions(self):
        if self.action == "create":
            permission_classes = [AuthOnlyPermission]
        else:
            permission_classes = []

        return [permission() for permission in permission_classes]

    def get_object(self):
        pk = self.kwargs["pk"]
        # treat pk as followee_id
        return get_object_or_404(UserFollow, follower=self.request.user, followee=pk)

    def get_queryset(self):
        follower = self.request.user

        if "user" in self.request.query_params:
            follower_spec = UserSpecifier.parse_str(self.request.query_params["user"])
            follower = follower_spec.get_user_model()
            if follower is None:
                raise Http404("user not found")

        return UserFollow.objects.filter(follower=follower).order_by("-created_at")

    def get_serializer_context(self):
        return {
            "request": self.request,
        }


class UserFollowerViewset(
    JsonldAwareMixin,
    mixins.ListModelMixin,
    viewsets.GenericViewSet,
):
    serializer_class = UserFollowerSerializer
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

    def get_object(self):
        pk = self.kwargs["pk"]
        # treat pk as followee_id
        return get_object_or_404(UserFollow, followee=self.request.user, follower=pk)

    def get_queryset(self):
        followee = self.request.user

        if "user" in self.request.query_params:
            followee_spec = UserSpecifier.parse_str(self.request.query_params["user"])
            followee = followee_spec.get_user_model()
            if followee is None:
                raise Http404("user not found")

        return UserFollow.objects.filter(followee=followee).order_by("-created_at")

    def get_serializer_context(self):
        return {
            "request": self.request,
        }


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
