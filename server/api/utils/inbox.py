from dataclasses import dataclass
from typing import Iterator, TypedDict, Union

from api.serializers.pub import PUBLIC_URI
from api.utils.get_id import get_user_id, make_followers_id
from api.utils.posts.privacy import PostPrivacy

from ..models import Post, PostMention, User, UserFollow


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
            UserFollow.objects.filter(
                followee_id=target.followed_user.id,
                follower__host__isnull=False,
            )
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
        return [_find_shared_or_inbox(target.target_user)]
    else:
        raise ValueError("invalid target type: " + str(target))


def _find_shared_or_inbox(target: User) -> str | None:
    if (
        hasattr(target, "remote_user_info")
        and target.remote_user_info.shared_inbox is not None
    ):
        return target.remote_user_info.shared_inbox
    else:
        return target.inbox


class PostToCcList(TypedDict):
    to: list[str]
    cc: list[str]
    target_inboxes: list[str]


class TargetList(TypedDict):
    to: list[str]
    cc: list[str]
    delivery_targets: list[DeliveryTarget]


def calculate_to_and_cc(post: Post) -> TargetList:
    """
    Calculate 'to' and 'cc' lists based on the post's privacy settings and mentions.
    """
    if post.privacy == 0:
        # public
        to = [PUBLIC_URI]
        cc = [make_followers_id(post.poster)]
        delivery_targets = [FollowerTarget(followed_user=post.poster)]
    elif post.privacy == 1:
        # unlisted
        to = [make_followers_id(post.poster)]
        cc = [PUBLIC_URI]
        delivery_targets = [FollowerTarget(followed_user=post.poster)]
    elif post.privacy == 2:
        # followers only
        to = [make_followers_id(post.poster)]
        cc = []
        delivery_targets = [FollowerTarget(followed_user=post.poster)]
    elif post.privacy == 3:
        # private only (see mentions below)
        to = []
        cc = []
        delivery_targets = []
    else:
        raise ValueError("invalid privacy")

    # process mentions
    mentions: Iterator[PostMention] = post.mentions.all()
    for mention in mentions:
        to.append(get_user_id(mention.target_user))
        delivery_targets.append(UserTarget(target_user=mention.target_user))

    return {"to": to, "cc": cc, "delivery_targets": delivery_targets}


def calculate_target_inboxes(delivery_targets: list[FollowerTarget]) -> list[str]:
    """
    Calculate 'target_inboxes' based on delivery targets.
    """
    target_inboxes = find_common_inbox(delivery_targets)
    return target_inboxes


def make_to_and_cc(post: Post) -> PostToCcList:
    result = calculate_to_and_cc(post)
    target_inboxes = calculate_target_inboxes(result["delivery_targets"])

    return {
        "to": result["to"],
        "cc": result["cc"],
        "target_inboxes": target_inboxes,
    }


@dataclass
class InferredPrivacy:
    privacy: PostPrivacy
    # TODO: include information about targeted individuals (e.g. mentioned users)


def infer_privacy(to: list[str], cc: list[str]) -> InferredPrivacy:
    # TODO: make it more robust
    if PUBLIC_URI in to:
        return InferredPrivacy(privacy=PostPrivacy.PUBLIC)
    elif PUBLIC_URI in cc:
        return InferredPrivacy(privacy=PostPrivacy.UNLISTED)

    # heauristic: if url ends with "/followers", it is targeted to followers
    for t in to:
        if t.endswith("/followers"):
            return InferredPrivacy(privacy=PostPrivacy.FOLLOWERS)

    return InferredPrivacy(privacy=PostPrivacy.PRIVATE)
