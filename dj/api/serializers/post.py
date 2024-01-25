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


class PostNotFoundError(serializers.ValidationError):
    def __init__(self, msg):
        super().__init__(msg)


class CreatePostSerializer(serializers.ModelSerializer):
    reply_to_id = serializers.PrimaryKeyRelatedField(
        queryset=Post.objects.all(), required=False
    )
    repost_of_id = serializers.PrimaryKeyRelatedField(
        queryset=Post.objects.all(), required=False
    )

    def create(self, validated_data):
        poster = self.context["request"].user
        if not poster:
            raise serializers.ValidationError("User not authenticated")
        # if anonymous
        if not poster.id:
            raise serializers.ValidationError("User not authenticated")

        post = Post.objects.create(poster=poster, **validated_data)

        return post

    class Meta:
        model = Post
        fields = ["content", "privacy", "reply_to_id", "repost_of_id"]
