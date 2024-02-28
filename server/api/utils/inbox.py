from dataclasses import dataclass
from typing import Union

from ..models import User, UserFollow


@dataclass
class FollowerTarget:
    followed_user: User


@dataclass
class UserTarget:
    target_user: User


DeliveryTarget = Union[FollowerTarget, UserTarget]


def find_common_inbox(targets: list[DeliveryTarget]) -> list[str]:
    """
    Find common inbox for the given targets considering sharedInboxes
    """
    inboxes = []
    inbox_set = set()  # for fast lookup

    for target in targets:
        new_inboxes = _find_best_inbox(target)
        for new_inbox in new_inboxes:
            if new_inbox not in inbox_set:
                inboxes.append(new_inbox)
                inbox_set.add(new_inbox)

    return inboxes


def _find_best_inbox(target: DeliveryTarget) -> list[str]:
    if isinstance(target, FollowerTarget):
        follows = (
            UserFollow.objects.filter(followee_id=target.followed_user.id)
            .select_related("follower")
            .all()
        )
        inboxes = []
        inbox_set = set()
        for follow in follows:
            inbox = _find_shared_or_inbox(follow.follower)
            if inbox is not None and inbox not in inbox_set:
                inboxes.append(inbox)
                inbox_set.add(inbox)
        return inboxes
    elif isinstance(target, UserTarget):
        return _find_shared_or_inbox(target.target_user)


def _find_shared_or_inbox(target: User) -> str | None:
    if target.remote_user_info and target.remote_user_info.shared_inbox:
        return target.remote_user_info.shared_inbox
    else:
        return target.inbox
