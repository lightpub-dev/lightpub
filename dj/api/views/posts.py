from ..serializers.post import CreatePostSerializer
from ..models import Post
from rest_framework import generics, status
from ..auth import TokenAuth, AuthOnlyPermission


class CreatePostView(generics.CreateAPIView):
    permission_classes = [AuthOnlyPermission]
    serializer_class = CreatePostSerializer
    queryset = Post.objects.all()
