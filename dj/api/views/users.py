from rest_framework import generics, mixins, status, views, viewsets
from rest_framework.response import Response

from api.utils.users import UserSpecifier
from ..auth import AuthOnlyPermission, NoAuthPermission
from ..models import User, UserFollow
from ..serializers.user import (
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


class UserFollowViewset(
    mixins.CreateModelMixin, mixins.DestroyModelMixin, generics.GenericAPIView
):
    def get_permissions(self):
        if self.action == "create":
            permission_classes = [AuthOnlyPermission]
        else:
            permission_classes = []

        return [permission() for permission in permission_classes]

    def get_queryset(self):
        return UserFollow.objects.filter(follower=self.request.user)
