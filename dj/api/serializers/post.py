from rest_framework import serializers

from api.models import User, UserFollow, UserProfileLabel, UserToken, Post


class PostAuthorSerializer(serializers.ModelSerializer):
    class Meta:
        model = User
        fields = ["id", "username", "host", "nickname"]


class PostEntrySerializer(serializers.ModelSerializer):
    author = PostAuthorSerializer()
    reposted_by_me = serializers.BooleanField(allow_null=True)
    favorited_by_me = serializers.BooleanField(allow_null=True)
    bookmarked_by_me = serializers.BooleanField(allow_null=True)
    repost_count = serializers.IntegerField(allow_null=True)
    favorite_count = serializers.IntegerField(allow_null=True)
    reply_count = serializers.IntegerField(allow_null=True)
    quote_count = serializers.IntegerField(allow_null=True)
    reactions = serializers.DictField(child=serializers.IntegerField(), allow_null=True)

    class Meta:
        model = Post
        fields = ["id", "content", "created_at", "privacy"]


class CreatePostSerializer(serializers.ModelSerializer):
    def save(self, validated_data):
        """
        Additional `poster_id` field is required to save a post.
        Accept `reply_to_id` or `repost_of_id` or `
        """

    class Meta:
        model = Post
        fields = ["content", "privacy"]
