from django.http.response import HttpResponse

from api.models import UserToken


def auth_middleware(get_response):
    def middleware(request):
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
            return get_response(request)
        except UserToken.DoesNotExist:
            return HttpResponse("Authorization failed", status=401)

    return middleware
