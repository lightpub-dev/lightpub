from rest_framework.decorators import api_view
from rest_framework.response import Response
from lightpub.settings import VERSION
from api.models import User
from rest_framework.reverse import reverse


def get_total_users():
    # count of local users
    return User.objects.filter(host=None).count()


def _gen_data():
    return {
        "version": "2.1",
        "software": {
            "name": "lightpub",
            "version": VERSION,
        },
        "protocol": [
            "activitypub",
        ],
        "services": {"inbound": [], "outbound": []},
        "openRegistrations": True,
        "usage": {
            "users": {
                "total": get_total_users(),
            }
        },
        "metadata": {},
    }


@api_view(["GET"])
def version_2_1(request):
    return Response(status=200, data=_gen_data())


@api_view(["GET"])
def version_2_0(request):
    return Response(status=200, data=_gen_data())


@api_view(["GET"])
def nodeinfo(request):
    return Response(
        status=200,
        data={
            "links": [
                {
                    "rel": "http://nodeinfo.diaspora.software/ns/schema/2.1",
                    "href": reverse("api:nodeinfo-2.1", request=request),
                },
                {
                    "rel": "http://nodeinfo.diaspora.software/ns/schema/2.0",
                    "href": reverse("api:nodeinfo-2.0", request=request),
                },
            ]
        },
    )
