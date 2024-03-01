from rest_framework import renderers
from rest_framework.utils import encoders, json


class XrdXmlRenderer(renderers.BaseRenderer):
    media_type = "application/xrd+xml"
    format = "xrd+xml"

    def render(self, data, accepted_media_type=None, renderer_context=None):
        # Only GET requests are considered
        # no need to actually parse the body as xml
        body = json.dumps(data, cls=encoders.JSONEncoder, indent=2)
        return body.encode("utf-8")
