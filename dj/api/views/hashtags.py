from ..serializers.post import HashtagSerializer, PostSerializer
from ..models import Post, PostHashtag
from rest_framework import generics, mixins
from rest_framework.response import Response
from django.db.models import Count
from api.pagination import MyPagination
import datetime


class PopularHashtagPage(MyPagination):
    page_size = 10
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
