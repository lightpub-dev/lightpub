from drf_extra_fields.relations import PresentablePrimaryKeyRelatedField
from rest_framework import serializers

from ..models import PostBookmark, PostFavorite, PostReaction
from .post import PostSerializer, ReplyToIdField, visible_posts
from .user import SimpleUserSerializer


class FavoritablePostField(serializers.PrimaryKeyRelatedField):
    def get_queryset(self):
        user = self.context["request"].user

        if not user.id:
            return []
        return visible_posts(user)


class PostFavoriteSerializer(serializers.ModelSerializer):
    post = PostSerializer(read_only=True)
    post_id = FavoritablePostField(write_only=True, allow_null=False, required=True)
    user = SimpleUserSerializer(read_only=True)
    created_at = serializers.DateTimeField(read_only=True)

    def validate_post_id(self, value):
        # unique check
        user = self.context["request"].user
        if PostFavorite.objects.filter(user=user, post=value).exists():
            raise serializers.ValidationError("Already favorited")
        return value

    def create(self, validated_data):
        user = self.context["request"].user
        if not user.id:
            raise serializers.ValidationError("User not authenticated")
        validated_data["post"] = validated_data["post_id"]
        del validated_data["post_id"]
        return PostFavorite.objects.create(user=user, **validated_data)

    class Meta:
        model = PostFavorite
        fields = ["post_id", "post", "user", "created_at"]


class PostBookmarkSerializer(serializers.ModelSerializer):
    post = PostSerializer(read_only=True)
    post_id = FavoritablePostField(write_only=True, allow_null=False, required=True)
    user = SimpleUserSerializer(read_only=True)
    created_at = serializers.DateTimeField(read_only=True)

    def validate_post_id(self, value):
        # unique check
        user = self.context["request"].user
        if PostBookmark.objects.filter(user=user, post=value).exists():
            raise serializers.ValidationError("Already favorited")
        return value

    def create(self, validated_data):
        user = self.context["request"].user
        if not user.id:
            raise serializers.ValidationError("User not authenticated")
        validated_data["post"] = validated_data["post_id"]
        del validated_data["post_id"]
        return PostBookmark.objects.create(user=user, **validated_data)

    class Meta:
        model = PostBookmark
        fields = ["post_id", "post", "user", "created_at"]


class ReactionablePostField(PresentablePrimaryKeyRelatedField):
    def get_queryset(self):
        user = self.context["request"].user

        if not user.id:
            return []
        return visible_posts(user)


class PostReactionSerializer(serializers.ModelSerializer):
    post = ReactionablePostField(
        presentation_serializer=PostSerializer,
    )
    user = SimpleUserSerializer(read_only=True)

    def create(self, validated_data):
        user = self.context["request"].user
        if not user.id:
            raise serializers.ValidationError("User not authenticated")

        return PostReaction.objects.create(user=user, **validated_data)

    def validate(self, data):
        # unique check
        user = self.context["request"].user
        if PostReaction.objects.filter(
            user=user, post=data["post"], emoji=data["emoji"]
        ).exists():
            raise serializers.ValidationError("Already reacted")

        return data

    class Meta:
        model = PostReaction
        fields = ["post", "user", "emoji", "created_at"]
