import re
import uuid

import bcrypt
from rest_framework import serializers

from api.models import User, UserFollow, UserProfileLabel, UserToken

USERNAME_RE = re.compile(r"^[a-zA-Z0-9_-]{3,60}$")


def username_validator(username):
    if not USERNAME_RE.match(username):
        raise serializers.ValidationError("does not match username pattern")


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
