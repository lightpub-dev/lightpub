from rest_framework import renderers
import json


class JsonldRenderer(renderers.BaseRenderer):
    media_type = "application/ld+json"
    format = "jsonld"

    def render(self, data, accepted_media_type=None, renderer_context=None):
        body = json.dumps(data, indent=2)
        return body.encode("utf-8")
