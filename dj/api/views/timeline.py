from ..models import Post
from ..serializers.post import PostSerializer
from rest_framework import generics, mixins
from rest_framework.response import Response
from ..auth import AuthOnlyPermission
from django.db.models import Q


class TimelineView(generics.ListAPIView):
    serializer_class = PostSerializer
    permission_classes = [AuthOnlyPermission]

    def get_queryset(self):
        user = self.request.user
        if not user.id:
            return []
        posts = Post.objects.filter(
            Q(privacy=0)  # public
            | Q(
                privacy=2, poster__followers__follower=user
            )  # unlisted  # followers only
            | Q(privacy=3, poster=user)  # private myself
        ).order_by("-created_at")
        return posts
