from rest_framework import serializers

from api.models import User, UserFollow, UserProfileLabel, UserToken, Post
from typing import Any, cast
from django.db.models import Q


class PostAuthorSerializer(serializers.ModelSerializer):
    class Meta:
        model = User
        fields = ["id", "username", "host", "nickname"]


class PostNotFoundError(serializers.ValidationError):
    def __init__(self, msg):
        super().__init__(msg)


def visible_posts(user: User):
    posts = Post.objects.filter(
        Q(privacy=0)
        | Q(privacy=1)  # public
        | Q(privacy=2, poster__followers__follower=user)  # unlisted  # followers only
        | Q(privacy=3, poster=user)  # private myself
    )
    return posts


class ReplyToIdField(serializers.PrimaryKeyRelatedField):
    def get_queryset(self):
        user = cast(User | Any, self.context["request"].user)

        if not user.id:
            return []

        # posts that are visible to the user
        return visible_posts(user)


class RepostOfIdField(serializers.PrimaryKeyRelatedField):
    def get_queryset(self):
        # posts that are visible to the user
        posts = Post.objects.filter(privacy__in=[0, 1])  # public  # unlisted
        return posts


class PostSerializer(serializers.ModelSerializer):
    id = serializers.UUIDField(read_only=True)
    author = PostAuthorSerializer(read_only=True, source="poster")
    reply_to_id = ReplyToIdField(allow_null=True, required=False, default=None)
    repost_of_id = RepostOfIdField(allow_null=True, required=False, default=None)

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
        fields = ["id", "author", "content", "privacy", "reply_to_id", "repost_of_id"]
