from rest_framework import serializers
from ..models import PostFavorite, PostBookmark
from .post import PostSerializer, visible_posts
from .user import SimpleUserSerializer


class FavoritablePostField(serializers.PrimaryKeyRelatedField):
    def get_queryset(self):
        user = self.context["request"].user

        if not user.id:
            return []
        return visible_posts(user)


class PostFavoriteSerializer(serializers.ModelSerializer):
    id = serializers.UUIDField(read_only=True)
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
        fields = ["id", "post_id", "post", "user", "created_at"]


class PostBookmarkSerializer(serializers.ModelSerializer):
    id = serializers.UUIDField(read_only=True)
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
        fields = ["id", "post_id", "post", "user", "created_at"]
