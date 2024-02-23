from rest_framework import renderers
from rest_framework.utils import encoders, json


class JsonldRenderer(renderers.BaseRenderer):
    media_type = 'application/ld+json; profile="https://www.w3.org/ns/activitystreams"'
    format = "jsonld"

    def render(self, data, accepted_media_type=None, renderer_context=None):
        body = json.dumps(data, cls=encoders.JSONEncoder, indent=2)
        return body.encode("utf-8")


class ActivityJsonRenderer(JsonldRenderer):
    media_type = "application/activity+json"


class WebfingerRenderer(JsonldRenderer):
    media_type = "application/jrd+json"
    format = "jrd+json"
