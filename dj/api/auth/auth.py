from rest_framework.authentication import BaseAuthentication
from rest_framework.exceptions import AuthenticationFailed
from ..models import UserToken


class CookieAuth(BaseAuthentication):
    def authenticate(self, request):
        # read bearer token
        token = request.COOKIES.get("auth_token", None)

        if token is None:
            return None

        # check token
        try:
            user_token = UserToken.objects.select_related("user").get(token=token)
            return user_token.user, None
        except UserToken.DoesNotExist:
            return AuthenticationFailed("Authorization failed")


class TokenAuth(BaseAuthentication):
    def authenticate(self, request):
        # read bearer token
        token = request.headers.get("Authorization", None)

        if token is None:
            return None

        if token.startswith("Bearer "):
            token = token[7:]

        # check token
        try:
            user_token = UserToken.objects.select_related("user").get(token=token)
            return user_token.user, None
        except UserToken.DoesNotExist:
            return AuthenticationFailed("Authorization failed")
