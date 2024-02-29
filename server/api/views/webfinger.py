from django.urls import reverse
from rest_framework.renderers import BrowsableAPIRenderer, JSONRenderer
from rest_framework.response import Response
from rest_framework.views import APIView

from api.auth.permission import NoAuthPermission
from api.jsonld.renderer import ActivityJsonRenderer, JsonldRenderer, WebfingerRenderer
from api.models import User
from api.serializers.webfinger import UserSerializer
from lightpub.settings import HOSTNAME


def parse_resource(resource: str) -> User | None:
    if not resource.startswith("acct:"):
        return None
    resource = resource[5:]
    if "@" not in resource:
        return None

    username, domain = resource.split("@", maxsplit=2)

    if domain != HOSTNAME:
        return None

    user = User.objects.filter(username=username).first()
    return user


class WebFingerAcctView(APIView):
    permission_classes = [NoAuthPermission]
    renderer_classes = [
        WebfingerRenderer,
        JSONRenderer,
        JsonldRenderer,
        ActivityJsonRenderer,
        BrowsableAPIRenderer,
    ]

    def get(self, request):
        resource_query = request.query_params.get("resource", None)
        if not resource_query:
            return Response(data="Resource query is required", status=400)

        user = parse_resource(resource_query)
        if not user:
            return Response(data="User not found", status=404)

        user_url = request.build_absolute_uri(
            reverse("api:user-detail", kwargs={"pk": user.id})
        )
        user_url_by_username = request.build_absolute_uri(
            reverse("api:user-detail", kwargs={"pk": "@" + user.username})
        )

        serializer = UserSerializer(
            {
                "aliases": [user_url, user_url_by_username],
                "links": [
                    {
                        "rel": "self",
                        "type": 'application/ld+json; profile="https://www.w3.org/ns/activitystreams"',  # noqa: E501
                        "href": user_url,
                    }
                ],
                "subject": resource_query,
            }
        )

        return Response(
            data=serializer.data,
            status=200,
        )
