from django.http.response import HttpResponse

from api.models import UserToken


class AuthMiddleware:
    def __init__(self, get_response):
        self.get_response = get_response

    def __call__(self, request):
        # read bearer token
        token = request.headers.get("Authorization", None)

        if token is None:
            return HttpResponse("No auth token provided", status=401)

        if token.startswith("Bearer "):
            token = token[7:]

        # check token
        try:
            user_token = UserToken.objects.select_related("user").get(token=token)
            request.user = user_token.user
            return self.get_response(request)
        except UserToken.DoesNotExist:
            return HttpResponse("Authorization failed", status=401)
