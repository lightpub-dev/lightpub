import re

from api.models import Post, User
from lightpub.settings import HOSTNAME, HTTP_SCHEME


def get_user_public_key_id(user: User) -> str:
    # TODO: too fragile
    return f"{HTTP_SCHEME}://{HOSTNAME}/api/users/{user.id}#main-key"


def get_user_id(user: User) -> str:
    # TODO: too fragile
    return f"{HTTP_SCHEME}://{HOSTNAME}/api/users/{user.id}"


def get_post_id(post: Post) -> str:
    # TODO: too fragile
    return f"{HTTP_SCHEME}://{HOSTNAME}/api/posts/{post.id}"


def extract_local_user_id(uri: str) -> str | None:
    # TODO: too fragile
    m = re.match(rf"{HTTP_SCHEME}://{HOSTNAME}/api/users/([a-f\d\-]+)", uri)
    if m is None:
        return None
    return m.group(1)


def extract_local_post_id(uri: str) -> str | None:
    # TODO: too fragile
    m = re.match(rf"{HTTP_SCHEME}://{HOSTNAME}/api/posts/([a-f\d\-]+)", uri)
    if m is None:
        return None
    return m.group(1)


def make_followers_id(user: User) -> str:
    # TODO: too fragile
    return f"{HTTP_SCHEME}://{HOSTNAME}/api/followers/?user={user.id}"
