from rest_framework import serializers

from ..models import User


class UserSpecifier:
    def __init__(
        self,
        user_id: str | None = None,
        username_and_host: tuple[str, str | None] | None = None,
    ):
        self.user_id = user_id
        self.username_and_host = username_and_host

        if user_id is None and username_and_host is None:
            raise ValueError("user_id and username_and_host cannot both be None")
        elif user_id is not None and username_and_host is not None:
            raise ValueError("user_id and username_and_host cannot both be not None")

    @classmethod
    def parse_str(cls, user_spec: str):
        if "@" not in user_spec:
            return cls(user_id=user_spec)

        if not user_spec.startswith("@"):
            raise ValueError("user_spec must start with @ if not specifying user_id")

        username_and_host = user_spec[1:].split("@", maxsplit=2)
        if len(username_and_host) == 1:
            return cls(username_and_host=(username_and_host[0], None))
        elif len(username_and_host) == 2:
            return cls(username_and_host=(username_and_host[0], username_and_host[1]))

        raise ValueError("invalid user_spec")

    def get_user_model(self) -> User | None:
        if self.user_id is not None:
            return User.objects.filter(id=self.user_id).first()
        elif self.username_and_host is not None:
            return User.objects.filter(
                username=self.username_and_host[0], host=self.username_and_host[1]
            ).first()
        else:
            raise ValueError("user_id and username_and_host cannot both be None")

    def __str__(self):
        if self.user_id is not None:
            return self.user_id
        elif self.username_and_host is not None:
            if self.username_and_host[1] == "":
                return f"@{self.username_and_host[0]}"
            return f"@{self.username_and_host[0]}@{self.username_and_host[1]}"
        else:
            raise ValueError("user_id and username_and_host cannot both be None")


class UserSpecifierPath:
    regex = r"[a-zA-Z0-9_\-@]+"

    def to_python(self, value):
        try:
            return UserSpecifier.parse_str(value)
        except ValueError as e:
            raise ValueError("Invalid user specifier") from e

    def to_url(self, value: UserSpecifier):
        return str(value)


class UserSpecifierSerializer(serializers.CharField):
    def to_internal_value(self, data):
        try:
            return UserSpecifier.parse_str(data)
        except ValueError as e:
            raise serializers.ValidationError("Invalid user specifier") from e

    def to_representation(self, value):
        return str(value)
