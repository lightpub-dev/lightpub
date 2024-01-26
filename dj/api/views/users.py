from django.http import Http404
from django.shortcuts import get_object_or_404
from rest_framework import generics, mixins, status, views, viewsets
from rest_framework.response import Response

from api.utils.users import UserSpecifier
from ..auth import AuthOnlyPermission, NoAuthPermission
from ..models import User, UserFollow
from ..serializers.user import (
    DetailedUserSerializer,
    LoginSerializer,
    RegisterSerializer,
    login_and_generate_token,
    UserFollowSerializer,
)


# Create your views here.
class RegisterView(generics.CreateAPIView):
    permission_classes = [NoAuthPermission]
    serializer_class = RegisterSerializer
    queryset = User.objects.all()


class LoginView(views.APIView):
    permission_classes = [NoAuthPermission]
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
    mixins.RetrieveModelMixin,
    mixins.ListModelMixin,
    mixins.UpdateModelMixin,
    viewsets.GenericViewSet,
):
    permission_classes = [NoAuthPermission]
    queryset = User.objects.all()
    serializer_class = DetailedUserSerializer

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
    mixins.CreateModelMixin,
    mixins.DestroyModelMixin,
    mixins.ListModelMixin,
    viewsets.GenericViewSet,
):
    serializer_class = UserFollowSerializer

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
    mixins.ListModelMixin,
    viewsets.GenericViewSet,
):
    serializer_class = UserFollowSerializer
    permission_classes = []

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
