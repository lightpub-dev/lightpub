from rest_framework import pagination
from rest_framework import serializers
from rest_framework.utils.urls import replace_query_param, remove_query_param


class CollectionSerializer(serializers.Serializer):
    ctx = serializers.ReadOnlyField(default="https://www.w3.org/ns/activitystreams")
    type = serializers.ReadOnlyField(default="Collection")
    totalItems = serializers.IntegerField(read_only=True)
    items = serializers.ListField(read_only=True, required=False)
    first = serializers.URLField(read_only=True, required=False)
    last = serializers.URLField(read_only=True, required=False)

    def get_fields(self):
        fields = super().get_fields()
        fields["@context"] = fields.pop("ctx")

        return fields


class CollectionPageSerializer(CollectionSerializer):
    next = serializers.URLField(read_only=True, required=False)
    prev = serializers.URLField(read_only=True, required=False)


class OrderedCollectionSerializer(serializers.Serializer):
    ctx = serializers.ReadOnlyField(default="https://www.w3.org/ns/activitystreams")
    type = serializers.ReadOnlyField(default="OrderedCollection")
    totalItems = serializers.IntegerField(read_only=True)
    orderedItems = serializers.ListField(read_only=True, required=False)
    first = serializers.URLField(read_only=True, required=False)
    last = serializers.URLField(read_only=True, required=False)

    def get_fields(self):
        fields = super().get_fields()
        fields["@context"] = fields.pop("ctx")

        return fields


class OrderedCollectionPageSerializer(OrderedCollectionSerializer):
    next = serializers.URLField(read_only=True, required=False)
    prev = serializers.URLField(read_only=True, required=False)


class CollectionPagination(pagination.BasePagination):
    page_size = 10
    _collection_serializer = CollectionSerializer
    _page_serializer = CollectionPageSerializer
    page_param = "page"

    def __init__(self) -> None:
        self.current_page = None
        self.next_page = None
        self.total_count = None
        self._request = None

    def get_next_link(self):
        if self.next_page is None:
            return None

        url = self._request.build_absolute_uri()
        return replace_query_param(url, self.page_param, self.next_page)

    def get_first_link(self):
        url = self._request.build_absolute_uri()
        return replace_query_param(url, self.page_param, 1)

    def get_partof(self):
        url = self._request.build_absolute_uri()
        return remove_query_param(url, self.page_param)

    def paginate_queryset(self, queryset, request, view=None):
        self._request = request

        page_param = request.query_params.get("page", None)

        self.current_page = page_param

        # page without page parameter is a link to the first page
        if page_param is None:
            self.next_page = 1
            return []

        total_count = queryset.count()
        self.total_count = total_count

        page = int(page_param)
        page_size = self.page_size
        start = (page - 1) * page_size
        end = start + page_size

        self.next_page = page + 1

        return list(queryset[start:end])

    def get_paginated_response(self, data):
        if self.current_page is None:
            return self._collection_serializer(
                {
                    "totalItems": self.total_count,
                    "items": data,
                    "first": self.get_first_link(),
                }
            ).data
        else:
            return self._page_serializer(
                {
                    "totalItems": self.total_count,
                    "items": data,
                    "first": self.get_first_link(),
                    "next": self.get_next_link(),
                    "partOf": self.get_partof(),
                }
            ).data


class OrderedCollectionPagination(CollectionPagination):
    _collection_serializer = OrderedCollectionSerializer
