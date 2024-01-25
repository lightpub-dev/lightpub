from rest_framework import generics, mixins, status, views
from rest_framework.response import Response

from api.utils.users import UserSpecifier
from ..auth import AuthOnlyPermission, NoAuthPermission
from ..models import User, UserToken
from ..serializers.user import (
    LoginSerializer,
    RegisterSerializer,
    login_and_generate_token,
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


# class ModifyFollowView(views.APIView):
#     def put(self, request, user_spec: UserSpecifier):
