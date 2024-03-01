from django.shortcuts import render
from django.urls import reverse
from rest_framework.renderers import BrowsableAPIRenderer, JSONRenderer
from rest_framework.response import Response
from rest_framework.views import APIView

from api.auth.permission import NoAuthPermission
from api.jsonld.renderer import ActivityJsonRenderer, JsonldRenderer, WebfingerRenderer
from api.models import User
from api.serializers.webfinger import UserSerializer
from api.xml.renderer import XrdXmlRenderer
from lightpub.settings import HOSTNAME, HTTP_SCHEME


def parse_resource(resource: str) -> User | None:
    if not resource.startswith("acct:"):
        return None
    resource = resource[5:]

    username_and_domain = resource.split("@", maxsplit=1)

    if len(username_and_domain) == 1:
        username = username_and_domain[0]
        domain = HOSTNAME
    else:
        username, domain = username_and_domain

    if domain != HOSTNAME:
        return None

    user = User.objects.filter(username=username).first()
    return user


class WebFingerAcctView(APIView):
    permission_classes = [NoAuthPermission]
    renderer_classes = [
        XrdXmlRenderer,
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

        if isinstance(request.accepted_renderer, XrdXmlRenderer):
            return render(
                request,
                "api/webfinger.xml",
                {"api_url": user_url, "subject": resource_query[5:]},
                content_type="application/xrd+xml",
            )

        serializer = UserSerializer(
            {
                "aliases": [user_url, user_url_by_username],
                "links": [
                    {
                        "rel": "self",
                        "type": "application/activity+json",  # noqa: E501
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


class HostMetaView(APIView):
    permission_classes = [NoAuthPermission]
    renderer_classes = [
        XrdXmlRenderer,
        JSONRenderer,
        JsonldRenderer,
        ActivityJsonRenderer,
        BrowsableAPIRenderer,
    ]

    def get(self, request):
        base_url = f"{HTTP_SCHEME}://{HOSTNAME}"
        return render(
            request,
            "api/host-meta.xml",
            {"base_url": base_url},
            content_type="application/xrd+xml",
        )
