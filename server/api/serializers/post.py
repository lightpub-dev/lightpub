import copy
from typing import Any, cast

from django.db.models import Count, Q
from django.urls import reverse
from rest_framework import serializers

from api.models import (
    Post,
    PostAttachment,
    PostMention,
    PostReaction,
    UploadedFile,
    User,
)
from api.utils.get_id import get_post_id, get_user_id
from api.utils.posts.db import CreatePostData, create_post
from api.utils.posts.privacy import PostPrivacy


class PostAuthorSerializer(serializers.ModelSerializer):
    avatar = serializers.SerializerMethodField()

    def get_avatar(self, obj):
        if not obj.avatar:
            return None

        if "request" not in self.context:
            return None

        request = self.context["request"]
        return request.build_absolute_uri(reverse("api:user-avatar", args=[obj.id]))

    class Meta:
        model = User
        fields = ["id", "username", "host", "nickname", "avatar"]


class PostNotFoundError(serializers.ValidationError):
    def __init__(self, msg):
        super().__init__(msg)


def visible_posts(user: User):
    posts = Post.objects.distinct().filter(
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


class UploadedFileSerializer(serializers.ModelSerializer):
    file = serializers.ImageField(write_only=True)

    def create(self, validated_data):
        uploader = self.context["request"].user

        return UploadedFile.objects.create(uploader=uploader, **validated_data)

    class Meta:
        model = UploadedFile
        fields = ["id", "uploader", "media_type", "created_at", "file"]
        read_only_fields = ["id", "uploader", "media_type", "created_at"]


class PostAttachmentSerializer(serializers.ModelSerializer):
    file = UploadedFileSerializer(read_only=True)
    url = serializers.SerializerMethodField()

    def get_url(self, attachment):
        if "request" not in self.context:
            return None
        request = self.context["request"]
        return request.build_absolute_uri(
            reverse("api:attachment", args=[attachment.id])
        )

    class Meta:
        model = PostAttachment
        fields = ["id", "file", "url"]
        read_only_fields = ["id", "file"]


class AttachedFileField(serializers.PrimaryKeyRelatedField):
    def get_queryset(self):
        # uploads by the user
        user = self.context["request"].user
        if not user.id:
            return []

        return UploadedFile.objects.filter(uploader=user)


class PostReactionInfoSerializer(serializers.Serializer):
    count = serializers.IntegerField()
    reacted_by_me = serializers.BooleanField(required=False, allow_null=True)


class MentionedUserSerializer(serializers.ModelSerializer):
    id = serializers.UUIDField(source="target_user.id", read_only=True)
    username = serializers.CharField(source="target_user.username", read_only=True)
    host = serializers.CharField(source="target_user.host", read_only=True)
    nickname = serializers.CharField(source="target_user.nickname", read_only=True)

    class Meta:
        model = PostMention
        fields = ["id", "username", "host", "nickname"]


class PostSerializer(serializers.ModelSerializer):
    id = serializers.UUIDField(read_only=True)
    author = PostAuthorSerializer(read_only=True, source="poster")
    reply_to_id = ReplyToIdField(allow_null=True, required=False, default=None)
    repost_of_id = RepostOfIdField(allow_null=True, required=False, default=None)

    reply_to = serializers.SerializerMethodField()
    repost_of = serializers.SerializerMethodField()
    external_uri = serializers.URLField(read_only=True, source="uri", allow_null=True)

    attached_uploads = AttachedFileField(
        allow_null=True, many=True, required=False, write_only=True
    )
    attached_files = serializers.SerializerMethodField()

    reply_count = serializers.SerializerMethodField()
    repost_count = serializers.SerializerMethodField()
    quote_count = serializers.SerializerMethodField()
    favorite_count = serializers.SerializerMethodField()

    reposted_by_me = serializers.SerializerMethodField()
    favorited_by_me = serializers.SerializerMethodField()
    bookmarked_by_me = serializers.SerializerMethodField()

    mentions = MentionedUserSerializer(many=True, read_only=True)
    reactions = serializers.SerializerMethodField()

    def validate_repost_of_id(self, repost_of_id):
        if repost_of_id is None:
            return None

        repost_target = repost_of_id
        if repost_target is None:
            raise serializers.ValidationError("Repost target not found")

        if repost_target.repost_of_id is not None:
            raise serializers.ValidationError("Cannot repost a repost")

        return repost_of_id

    def validate(self, data):
        # check double repost
        null_content = data.get("content", None) is None
        reposting = data.get("repost_of_id", None) is not None
        if null_content and reposting:
            # check if the user has already reposted the target
            user = self.context["request"].user
            if not user.id:
                raise serializers.ValidationError("User not authenticated")

            already_reposted = Post.objects.filter(
                poster=user, repost_of_id=data["repost_of_id"].id, content=None
            ).exists()
            if already_reposted:
                raise serializers.ValidationError("You cannot repost a post twice")

        return data

    def get_reply_to(self, post):
        if post.reply_to is None:
            return None
        return PostSerializer(post.reply_to, context=self.context).data

    def get_repost_of(self, post):
        if post.repost_of is None:
            return None

        repost_nest_level = self.context.get("nested_repost", 1)
        if repost_nest_level == 0:
            return None

        new_ctx = copy.copy(self.context)
        new_ctx["nested_repost"] = repost_nest_level - 1
        return PostSerializer(post.repost_of, context=new_ctx).data

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
        repost = post.reposts.filter(poster=user, content=None).first()
        if repost is None:
            return None
        return repost.id

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

    def get_attached_files(self, post):
        return PostAttachmentSerializer(
            many=True, context=self.context
        ).to_representation(post.attachments.all())

    def create(self, validated_data):
        poster = self.context["request"].user
        if not poster:
            raise serializers.ValidationError("User not authenticated")
        # if anonymous
        if not poster.id:
            raise serializers.ValidationError("User not authenticated")

        try:
            privacy = PostPrivacy(validated_data["privacy"])
        except ValueError:
            raise serializers.ValidationError("Invalid privacy value")

        repost_of_id = None
        if (
            "repost_of_id" in validated_data
            and validated_data["repost_of_id"] is not None
        ):
            repost_of_id = validated_data["repost_of_id"].id
        reply_to_id = None
        if (
            "reply_to_id" in validated_data
            and validated_data["reply_to_id"] is not None
        ):
            reply_to_id = validated_data["reply_to_id"].id
        attached_uploads = []
        if (
            "attached_uploads" in validated_data
            and validated_data["attached_uploads"] is not None
        ):
            attached_uploads = [
                upload.id for upload in validated_data["attached_uploads"]
            ]
        post_data = CreatePostData(
            poster=poster,
            content=validated_data.get("content", None),
            repost_of_id=repost_of_id,
            reply_to_id=reply_to_id,
            attached_uploads=attached_uploads,
            privacy=privacy,
        )
        return create_post(post_data)

    def get_reactions(self, post):
        reactions = (
            PostReaction.objects.filter(post=post)
            .values("emoji")
            .annotate(count=Count("user"))
            .order_by("-count")
        )

        user = self.context["request"].user
        reaction_count = {}
        for reaction in reactions:
            reaction_count[reaction["emoji"]] = {
                "count": reaction["count"],
            }
            if user.id:
                reacted_by_me = PostReaction.objects.filter(
                    post=post, user=user, emoji=reaction["emoji"]
                ).exists()
                reaction_count[reaction["emoji"]]["reacted_by_me"] = reacted_by_me

        return serializers.DictField(
            child=PostReactionInfoSerializer()
        ).to_representation(reaction_count)

    class Meta:
        model = Post
        fields = [
            "id",
            "author",
            "content",
            "privacy",
            "reply_to_id",
            "repost_of_id",
            "reply_to",
            "repost_of",
            "created_at",
            "reply_count",
            "repost_count",
            "quote_count",
            "favorite_count",
            "reposted_by_me",
            "favorited_by_me",
            "bookmarked_by_me",
            "attached_files",
            "attached_uploads",
            "mentions",
            "reactions",
            "external_uri",
        ]
        read_only_fields = ["created_at"]


class HashtagSerializer(serializers.Serializer):
    hashtag = serializers.CharField(max_length=255)
    recent_post_count = serializers.IntegerField()


class PostAddToListSerializer(serializers.Serializer):
    # no fields required
    pass
