from rest_framework import generics

from .renderer import JsonldRenderer


class JsonldAwareMixin(generics.GenericAPIView):
    def jsonld_requested(self) -> bool:
        return isinstance(self.request.accepted_renderer, JsonldRenderer)


class JsonldMixin(JsonldAwareMixin):
    normal_serializer_class = None
    jsonld_serializer_class = None

    def get_serializer_class(self):
        if self.jsonld_requested():
            return self.jsonld_serializer_class
        return self.normal_serializer_class
