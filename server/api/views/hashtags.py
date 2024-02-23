import datetime

from django.db.models import Count
from rest_framework import generics, mixins
from rest_framework.response import Response

from api.pagination import MyPagination

from ..models import Post, PostHashtag
from ..serializers.post import HashtagSerializer, PostSerializer


class PopularHashtagPage(MyPagination):
    def get_page_size(self, request):
        # check limit param
        limit = request.query_params.get("limit", None)
        if limit is not None:
            try:
                limit = int(limit)
                if limit < 0:
                    raise ValueError
                return limit
            except ValueError as e:
                raise ValueError("invalid limit") from e

        # default limit
        return 10

    ordering = "-recent_post_count"


class PopularHashtagsView(generics.ListAPIView):
    serializer_class = HashtagSerializer
    pagination_class = PopularHashtagPage

    def get_queryset(self):
        # count the number of posts with each hashtag within the last 24 hours
        # return the top 10 hashtags
        one_day_before = datetime.datetime.now() - datetime.timedelta(days=1)
        hashtags = (
            PostHashtag.objects.filter(post__created_at__gte=one_day_before)
            .values("hashtag")
            .annotate(recent_post_count=Count("post"))
        )
        return hashtags
