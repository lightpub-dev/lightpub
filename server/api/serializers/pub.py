from dataclasses import dataclass
from typing import Any, TypeGuard, TypeVar, Type, Union


def _qt(s: str) -> str:
    return f"https://www.w3.org/ns/activitystreams#{s}"


def _qt_map(d: dict, m: dict[str, str | tuple[str, "Node"]]) -> dict[str, Any]:
    return _qt_map2(d, {}, m)


def _qt_map2(
    d: dict, mandatory: dict[str, str], optional: dict[str, str]
) -> dict[str, Any]:
    dd = {}

    for k, v in mandatory.items():
        if not k.startswith("http"):
            k = _qt(k)
        if isinstance(v, tuple):
            vv = v[0]
            cls = v[1]
        else:
            vv = v
            cls = None
        if k in d:
            if cls:
                dd[vv] = cls.from_dict(d[k][0])
            else:
                dd[vv] = d[k][0]["@value"]
        else:
            raise MissingAttributeError(k, obj=d)

    for k, v in optional.items():
        if not k.startswith("http"):
            k = _qt(k)
        if isinstance(v, tuple):
            vv = v[0]
            cls = v[1]
        else:
            vv = v
            cls = None
        if k in d:
            if cls:
                dd[vv] = cls.from_dict(d[k][0])
            else:
                dd[vv] = d[k][0]["@value"]
        else:
            dd[vv] = None
    return dd


class MissingAttributeError(Exception):
    def __init__(self, missing_attribute: str, obj: dict | None = None) -> None:
        self.missing_attribute = missing_attribute
        self.obj = obj

    def __str__(self) -> str:
        return f"missing attribute {self.missing_attribute}: object {self.obj}"


class InvalidFormatError(Exception):
    def __init__(self, message: str) -> None:
        self.message = message


class ValidationError(Exception):
    def __init__(self, message: str) -> None:
        self.message = message


T = TypeVar("T")


@dataclass(kw_only=True)
class Node:
    id: str | None
    type: list[str] | None
    _source_obj: dict

    @classmethod
    def _build_from_dict(cls, d: dict) -> dict:
        id = d.get("@id", None)
        type = d.get("@type", None)
        return {"id": id, "type": type}

    @classmethod
    def from_dict(cls, d: dict) -> "Node":
        return cls(**cls._build_from_dict(d), _source_obj=d)

    def reparse(self, target_type: Type[T]) -> T:
        return target_type.from_dict(self._source_obj)

    def is_as_type(self, t: str) -> bool:
        as_t = _qt(t)
        return as_t in self.type


@dataclass(kw_only=True)
class Object(Node):
    """
    represents https://www.w3.org/ns/activitystreams#Object
    """

    as_name: str | None
    as_to: Union["Object", None]

    @classmethod
    def _build_from_dict(cls, d: dict) -> dict:
        return super()._build_from_dict(d) | _qt_map(
            d,
            {
                "name": "as_name",
                "to": ("as_to", Object),
            },
        )


@dataclass(kw_only=True)
class PublicKey(Object):
    as_owner: Object
    as_public_key_pem: str

    @classmethod
    def _build_from_dict(cls, d: dict) -> dict:
        d2 = super()._build_from_dict(d) | _qt_map(
            d,
            {
                "https://w3id.org/security#owner": ("as_owner", Object),
                "https://w3id.org/security#publicKeyPem": "as_public_key_pem",
            },
        )

        return d2


@dataclass(kw_only=True)
class Actor(Object):
    as_inbox: Object
    as_outbox: Object
    as_following: Object | None
    as_followers: Object | None
    as_liked: Object | None
    as_preferred_username: str | None
    as_public_key: PublicKey | None

    @classmethod
    def _build_from_dict(cls, d: dict) -> dict:
        return super()._build_from_dict(d) | _qt_map2(
            d,
            {
                "http://www.w3.org/ns/ldp#inbox": ("as_inbox", Object),
                "outbox": ("as_outbox", Object),
            },
            {
                "following": ("as_following", Object),
                "followers": ("as_followers", Object),
                "liked": ("as_liked", Object),
                "preferredUsername": "as_preferred_username",
                "https://w3id.org/security#publicKey": ("as_public_key", PublicKey),
            },
        )

    def validate(self) -> None:
        if self.type == "Person":
            raise ValidationError("type must be Person")
        if self.as_inbox is None:
            raise ValidationError("inbox is required")
        if self.as_outbox is None:
            raise ValidationError("outbox is required")


@dataclass(kw_only=True)
class Activity(Object):
    """
    represents https://www.w3.org/ns/activitystreams#Activity
    """

    as_actor: Object
    as_object: Object

    @classmethod
    def _build_from_dict(cls, d: dict) -> dict:
        return super()._build_from_dict(d) | _qt_map(
            d,
            {
                "actor": ("as_actor", Object),
                "object": ("as_object", Object),
            },
        )


@dataclass(kw_only=True)
class FollowActivity(Activity):
    def get_actor_id(self) -> str:
        return self.as_actor.id

    def get_object_id(self) -> str:
        return self.as_object.id


@dataclass(kw_only=True)
class UndoActivity(Activity):
    def get_actor_id(self) -> str:
        return self.as_actor.id

    def get_object_id(self) -> str:
        return self.as_object.id


def is_follow(obj: Object) -> TypeGuard[FollowActivity]:
    return obj.is_as_type("Follow")


def is_undo(obj: Object) -> TypeGuard[UndoActivity]:
    return obj.is_as_type("Undo")


def is_actor(obj: Object) -> TypeGuard[Actor]:
    return obj.is_as_type("Person")
