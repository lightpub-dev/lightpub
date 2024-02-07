import re
import uuid

import bcrypt
from rest_framework import serializers

from api.models import User, UserFollow, UserProfileLabel, UserToken
from django.urls import reverse
from django.core.exceptions import ObjectDoesNotExist
from api.utils.users import UserSpecifier
from django.db import transaction

USERNAME_RE = re.compile(r"^[a-zA-Z0-9\._-]{3,60}$")


def username_validator(username):
    if not USERNAME_RE.match(username):
        raise serializers.ValidationError("does not match username pattern")


class SimpleUserSerializer(serializers.ModelSerializer):
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
        fields = ["id", "username", "host", "nickname", "url", "avatar"]


class UserProfileLabelSerializer(serializers.ModelSerializer):
    class Meta:
        model = UserProfileLabel
        fields = ["key", "value"]


class AvatarIdField(serializers.PrimaryKeyRelatedField):
    def get_queryset(self):
        # my own uploads
        user = self.context["request"].user
        if not user.id:
            return []
        return user.uploaded_files.all()


class DetailedUserSerializer(serializers.ModelSerializer):
    password = serializers.CharField(required=True, min_length=4, write_only=True)

    url = serializers.SerializerMethodField("get_url")
    avatar = serializers.SerializerMethodField()
    labels = UserProfileLabelSerializer(many=True, required=False)

    n_posts = serializers.SerializerMethodField()
    n_followers = serializers.SerializerMethodField()
    n_followings = serializers.SerializerMethodField()
    is_following = serializers.SerializerMethodField()

    avatar_id = AvatarIdField(write_only=True, required=False, allow_null=True)

    def get_n_posts(self, obj):
        return obj.posts.filter(privacy__in=[0, 1]).count()

    def get_n_followers(self, obj):
        return obj.followers.count()

    def get_n_followings(self, obj):
        return obj.followings.count()

    def get_url(self, obj):
        if obj.url:
            return obj.url

        request = self.context["request"]
        full_url = request.build_absolute_uri(
            reverse("api:user-detail", kwargs={"pk": obj.id})
        )
        return full_url

    def get_avatar(self, obj):
        if not obj.avatar:
            return None

        if "request" not in self.context:
            return None

        request = self.context["request"]
        return request.build_absolute_uri(reverse("api:user-avatar", args=[obj.id]))

    def get_is_following(self, obj):
        if not self.context["request"].user.id:
            return None

        return UserFollow.objects.filter(
            follower=self.context["request"].user, followee=obj
        ).exists()

    def update(self, instance: User, validated_data):
        with transaction.atomic():
            if "labels" in validated_data:
                labels = validated_data["labels"]
                UserProfileLabel.objects.filter(user=instance).delete()
                for i, label in enumerate(labels):
                    UserProfileLabel.objects.create(user=instance, order=i, **label)
                del validated_data["labels"]
            if "avatar_id" in validated_data:
                if validated_data["avatar_id"] is None:
                    validated_data["avatar"] = None
                else:
                    validated_data["avatar"] = validated_data["avatar_id"]
                del validated_data["avatar_id"]
            if "password" in validated_data:
                bpasswd = bcrypt.hashpw(
                    validated_data["password"].encode("utf-8"), bcrypt.gensalt()
                )
                validated_data["bpassword"] = bpasswd.decode("utf-8")
                del validated_data["password"]
                UserToken.objects.filter(user=instance).delete()
            return super().update(instance, validated_data)

    class Meta:
        model = User
        fields = [
            "id",
            "password",
            "username",
            "host",
            "nickname",
            "url",
            "bio",
            "inbox",
            "outbox",
            "created_at",
            "labels",
            "n_posts",
            "n_followers",
            "n_followings",
            "is_following",
            "avatar",
            "avatar_id",
        ]
        extra_kwargs = {
            "id": {"read_only": True},
            "username": {"read_only": True},
            "host": {"read_only": True},
            "url": {"read_only": True},
            "inbox": {"read_only": True},
            "outbox": {"read_only": True},
            "created_at": {"read_only": True},
            "bio": {"required": False},
        }


class JsonldAttachmentSerializer(serializers.ModelSerializer):
    class Meta:
        model = UserProfileLabel
        fields = ["type", "key", "value"]

    type = serializers.ReadOnlyField(default="PropertyValue")


class JsonldDetailedUserSerializer(serializers.ModelSerializer):
    class Meta:
        model = User
        fields = [
            "ctx",
            "id",
            "inbox",
            "outbox",
            "following",
            "followers",
            "liked",
            "type",
            "attachment",
            "name",
        ]

    ctx = serializers.ReadOnlyField(default=["https://www.w3.org/ns/activitystreams"])
    id = serializers.SerializerMethodField()
    inbox = serializers.SerializerMethodField()
    outbox = serializers.SerializerMethodField()
    following = serializers.SerializerMethodField()
    followers = serializers.SerializerMethodField()
    liked = serializers.SerializerMethodField()
    type = serializers.ReadOnlyField(default="Person")
    name = serializers.CharField(source="nickname")

    attachment = JsonldAttachmentSerializer(many=True, required=False, source="labels")

    def get_id(self, obj):
        req = self.context["request"]
        return req.build_absolute_uri(reverse("api:user-detail", kwargs={"pk": obj.id}))

    def get_inbox(self, obj):
        if obj.inbox:
            return obj.inbox
        req = self.context["request"]
        return req.build_absolute_uri(
            reverse("api:inbox", kwargs={"user_spec": obj.id})
        )

    def get_outbox(self, obj):
        if obj.outbox:
            return obj.outbox
        req = self.context["request"]
        return req.build_absolute_uri(
            reverse("api:outbox", kwargs={"user_spec": obj.id})
        )

    def get_following(self, obj):
        req = self.context["request"]
        url = req.build_absolute_uri(
            reverse(
                "api:following-list",
            )
        )
        return f"{url}?user={obj.id}"

    def get_followers(self, obj):
        req = self.context["request"]
        url = req.build_absolute_uri(
            reverse(
                "api:follower-list",
            )
        )
        return f"{url}?user={obj.id}"

    def get_liked(self, obj):
        req = self.context["request"]
        url = req.build_absolute_uri(
            reverse(
                "api:favorite-list",
            )
        )
        return f"{url}?user={obj.id}"

    def get_fields(self):
        fields = super().get_fields()
        fields["@context"] = fields.pop("ctx")

        return fields


class RegisterSerializer(serializers.Serializer):
    username = serializers.CharField(
        required=True, max_length=60, min_length=3, validators=[username_validator]
    )
    password = serializers.CharField(required=True, min_length=4)
    nickname = serializers.CharField(required=True, max_length=200, min_length=3)

    def create(self, validated_data):
        username = validated_data["username"]

        if User.objects.filter(username=username).exists():
            raise serializers.ValidationError("username already exists")

        bpasswd = bcrypt.hashpw(
            validated_data["password"].encode("utf-8"), bcrypt.gensalt()
        )
        User.objects.create(
            username=username,
            nickname=validated_data["nickname"],
            bpassword=bpasswd.decode("utf-8"),
        )
        return validated_data


class LoginSerializer(serializers.Serializer):
    username = serializers.CharField(
        required=True, max_length=60, min_length=3, validators=[username_validator]
    )
    password = serializers.CharField(required=True, min_length=4)


def login_and_generate_token(username: str, password: str) -> str | None:
    try:
        user = User.objects.filter(username=username, host="", deleted_at=None).get()
        hashed_pw = user.bpassword.encode("utf-8")
        if bcrypt.checkpw(password.encode("utf-8"), hashed_pw):
            tokenUUID = uuid.uuid4().hex
            UserToken.objects.create(user=user, token=tokenUUID)
            return tokenUUID
        return None
    except User.DoesNotExist:
        return None


class FolloweeIdField(serializers.PrimaryKeyRelatedField):
    def get_queryset(self):
        user = self.context["request"].user

        if not user.id:
            return []

        return User.objects.exclude(id=self.context["request"].user.id)


class UserFollowSerializer(serializers.ModelSerializer):
    followee = SimpleUserSerializer(read_only=True)
    followee_spec = serializers.CharField(write_only=True, required=False)

    def validate_followee_spec(self, value):
        try:
            user_spec = UserSpecifier.parse_str(value)
        except ValueError as e:
            raise serializers.ValidationError(str(e))

        user = user_spec.get_user_model()
        if user is None:
            raise ObjectDoesNotExist("user not found")

        return user.id

    def create(self, validated_data):
        validated_data["followee_id"] = validated_data["followee_spec"]
        del validated_data["followee_spec"]

        return UserFollow.objects.create(
            follower=self.context["request"].user, **validated_data
        )

    def validate_followee_id(self, value):
        user = self.context["request"].user
        if UserFollow.objects.filter(follower=user, followee=value).exists():
            raise serializers.ValidationError("Already following")
        return value

    class Meta:
        model = UserFollow
        fields = ["followee", "followee_spec", "created_at"]
        extra_kwargs = {"created_at": {"read_only": True}}


class UserFollowerSerializer(serializers.ModelSerializer):
    follower = SimpleUserSerializer(read_only=True)

    class Meta:
        model = UserFollow
        fields = ["follower", "created_at"]
        extra_kwargs = {"created_at": {"read_only": True}}
