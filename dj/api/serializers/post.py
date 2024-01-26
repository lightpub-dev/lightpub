from rest_framework import serializers

from api.models import User, UserFollow, UserProfileLabel, UserToken, Post
from typing import cast
from django.db.models import Q


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


class ReplyToIdField(serializers.PrimaryKeyRelatedField):
    def get_queryset(self):
        user = cast(User, self.context["request"].user)
        # posts that are visible to the user
        posts = Post.objects.filter(
            Q(privacy=0)
            | Q(privacy=1)  # public
            | Q(
                privacy=2, poster__followers__follower=user
            )  # unlisted  # followers only
        )
        return posts


class RepostOfIdField(serializers.PrimaryKeyRelatedField):
    def get_queryset(self):
        # posts that are visible to the user
        posts = Post.objects.filter(privacy__in=[0, 1])  # public  # unlisted
        return posts


class CreatePostMixin(serializers.ModelSerializer):
    def create(self, validated_data):
        poster = self.context["request"].user
        if not poster:
            raise serializers.ValidationError("User not authenticated")
        # if anonymous
        if not poster.id:
            raise serializers.ValidationError("User not authenticated")

        post = Post.objects.create(poster=poster, **validated_data)

        return post

    id = serializers.UUIDField(read_only=True)
    author = PostAuthorSerializer(read_only=True, source="poster")

    class Meta:
        model = Post
        fields = ["id", "author", "content", "privacy", "reply_to_id", "repost_of_id"]


class CreatePostSerializer(CreatePostMixin):
    reply_to_id = ReplyToIdField(read_only=True)
    repost_of_id = RepostOfIdField(read_only=True)


class CreateReplySerializer(CreatePostMixin):
    reply_to_id = ReplyToIdField(allow_null=False, required=True)
    repost_of_id = RepostOfIdField(read_only=True)


class CreateRepostSerializer(serializers.ModelSerializer):
    reply_to_id = ReplyToIdField(read_only=True)
    repost_of_id = RepostOfIdField(allow_null=False, required=True)
    id = serializers.UUIDField(read_only=True)
    author = PostAuthorSerializer(read_only=True, source="poster")

    def create(self, validated_data):
        poster = self.context["request"].user
        if not poster:
            raise serializers.ValidationError("User not authenticated")
        # if anonymous
        if not poster.id:
            raise serializers.ValidationError("User not authenticated")

        post = Post.objects.create(poster=poster, content=None, **validated_data)

        return post

    class Meta:
        model = Post
        fields = ["id", "author", "privacy", "reply_to_id", "repost_of_id"]


class CreateQuoteSerializer(CreatePostMixin):
    reply_to_id = ReplyToIdField(read_only=True)
    repost_of_id = RepostOfIdField(allow_null=False, required=True)
