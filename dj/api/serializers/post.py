from rest_framework import serializers

from api.models import User, PostHashtag, Post
from typing import Any, cast
from django.db.models import Q
from django.db import transaction


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


def _find_hashtags(content: str) -> list[str]:
    hashtags = []
    in_hashtag = False
    hashtag_start_index = 0
    for i, ch in enumerate(content):
        if ch == "#":
            if not in_hashtag:
                in_hashtag = True
                hashtag_start_index = i
            else:
                # Reset if another # is found immediately after
                in_hashtag = False
        elif not ch.isalnum() and ch not in ["_", "-"]:
            # Word boundary detected
            if in_hashtag:
                hashtag = content[hashtag_start_index:i]
                if hashtag != "#":  # Ignore single '#' entries
                    hashtags.append(hashtag[1:])
                in_hashtag = False
        # Note: The Go code snippet handles end-of-content logic implicitly
    # Check if the content ends with a hashtag
    if in_hashtag:
        hashtag = content[hashtag_start_index:]
        if hashtag != "#":
            hashtags.append(hashtag[1:])

    # Remove duplicates
    unique_hashtags = list(set(hashtags))
    return unique_hashtags


class PostSerializer(serializers.ModelSerializer):
    id = serializers.UUIDField(read_only=True)
    author = PostAuthorSerializer(read_only=True, source="poster")
    reply_to_id = ReplyToIdField(allow_null=True, required=False, default=None)
    repost_of_id = RepostOfIdField(allow_null=True, required=False, default=None)

    reply_count = serializers.SerializerMethodField()
    repost_count = serializers.SerializerMethodField()
    quote_count = serializers.SerializerMethodField()
    favorite_count = serializers.SerializerMethodField()

    reposted_by_me = serializers.SerializerMethodField()
    favorited_by_me = serializers.SerializerMethodField()
    bookmarked_by_me = serializers.SerializerMethodField()

    def get_reply_count(self, post):
        return post.replies.count()

    def get_repost_count(self, post):
        return post.reposts.filter(content=None).count()

    def get_quote_count(self, post):
        return post.reposts.exclude(content=None).count()

    def get_favorite_count(self, post):
        return post.favorites.count()

    def get_reposted_by_me(self, post):
        user = self.context["request"].user
        if not user.id:
            return None
        return post.reposts.filter(poster=user, content=None).exists()

    def get_favorited_by_me(self, post):
        user = self.context["request"].user
        if not user.id:
            return None
        return post.favorites.filter(user=user).exists()

    def get_bookmarked_by_me(self, post):
        user = self.context["request"].user
        if not user.id:
            return None
        return post.bookmarks.filter(user=user).exists()

    def create(self, validated_data):
        poster = self.context["request"].user
        if not poster:
            raise serializers.ValidationError("User not authenticated")
        # if anonymous
        if not poster.id:
            raise serializers.ValidationError("User not authenticated")

        # begins transaction
        with transaction.atomic():
            post = Post.objects.create(poster=poster, **validated_data)

            # find hashtags
            if post.content is not None:
                hashtags = _find_hashtags(post.content)
            else:
                # fetch hashtags of the original post
                repost_of = Post.objects.get(id=post.repost_of_id)
                hashtags = PostHashtag.objects.filter(post=repost_of).values_list(
                    "hashtag", flat=True
                )

            for hashtag in hashtags:
                PostHashtag.objects.create(post=post, hashtag=hashtag)

        return post

    class Meta:
        model = Post
        fields = [
            "id",
            "author",
            "content",
            "privacy",
            "reply_to_id",
            "repost_of_id",
            "created_at",
            "reply_count",
            "repost_count",
            "quote_count",
            "favorite_count",
            "reposted_by_me",
            "favorited_by_me",
            "bookmarked_by_me",
        ]
        read_only_fields = ["created_at"]


class HashtagSerializer(serializers.Serializer):
    hashtag = serializers.CharField(max_length=255)
    recent_post_count = serializers.IntegerField()
