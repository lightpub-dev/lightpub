from django.db.models import Q
from rest_framework import generics

from ..auth import AuthOnlyPermission
from ..models import Post
from ..serializers.post import PostSerializer


class TimelineView(generics.ListAPIView):
    serializer_class = PostSerializer
    permission_classes = [AuthOnlyPermission]

    def get_queryset(self):
        user = self.request.user
        if not user.id:
            return []
        posts = (
            Post.objects.distinct()
            .filter(
                Q(poster=user)
                | Q(
                    privacy__in=[0, 2], poster__followers__follower=user
                )  # public or follower only
            )
            .order_by("-created_at")
        )
        return posts

    def get_serializer_context(self):
        return {
            "request": self.request,
        }
