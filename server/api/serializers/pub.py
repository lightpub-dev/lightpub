from dataclasses import dataclass
from typing import Any, TypeGuard


def _qt(s: str) -> str:
    return f"https://www.w3.org/ns/activitystreams#{s}"


def _qt_map(d: dict, m: dict[str, str]) -> dict[str, Any]:
    dd = {}
    for k, v in m.items():
        if k in d:
            dd[v] = d[k]
    return dd


@dataclass
class Node:
    id: str
    type: list[str] | None

    @classmethod
    def _build_from_dict(cls, d: dict) -> dict:
        return {"id": d["@id"], "type": d.get("@type", None)}

    @classmethod
    def from_dict(cls, d: dict) -> "Node":
        return cls(**cls._build_from_dict(d))

    def is_as_type(self, t: str) -> bool:
        as_t = _qt(t)
        return as_t in self.type


class Object(Node):
    """
    represents https://www.w3.org/ns/activitystreams#Object
    """

    as_name: str | None
    as_to: str | None

    @classmethod
    def _build_from_dict(cls, d: dict) -> dict:
        return super()._build_from_dict(d) | _qt_map(
            d,
            {
                "name": "as_name",
                "to": "as_to",
            },
        )


class Activity(Object):
    """
    represents https://www.w3.org/ns/activitystreams#Activity
    """

    as_actor: list[Object] | None
    as_object: list[Object] | None

    @classmethod
    def _build_from_dict(cls, d: dict) -> dict:
        return super()._build_from_dict(d) | _qt_map(
            d,
            {
                "actor": "as_actor",
                "object": "as_object",
            },
        )


class FollowActivity(Activity):
    def get_actor_id(self) -> str:
        return self.as_actor[0].id

    def get_object_id(self) -> str:
        return self.as_object[0].id


class UndoActivity(Activity):
    def get_actor_id(self) -> str:
        return self.as_actor[0].id

    def get_object_id(self) -> str:
        return self.as_object[0].id


def is_follow(obj: Object) -> TypeGuard[FollowActivity]:
    return obj.is_as_type("Follow")


def is_undo(obj: Object) -> TypeGuard[UndoActivity]:
    return obj.is_as_type("Undo")
