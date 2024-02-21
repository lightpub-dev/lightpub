from pyld import jsonld
import json
from rest_framework.parsers import BaseParser

# TODO: cache in redis
cache = {}


def caching_document_loader(url, options={}):
    loader = jsonld.requests_document_loader()
    if url in cache:
        return cache[url]
    resp = loader(url, options=options)
    cache[url] = resp
    return resp


jsonld.set_document_loader(caching_document_loader)


class ActivityJsonParser(BaseParser):
    media_type = "application/activity+json"

    def parse(self, stream, media_type=None, parser_context=None):
        return jsonld.expand(json.load(stream))
