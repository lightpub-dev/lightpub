from uuid import UUID

from celery import shared_task

from api.models import Post, User, UserFollowRequest
from api.utils.keygen import generate_key_pair

from .requester import get_requester


@shared_task
def fetch_remote_id(id: str) -> list:
    """
    Fetch remote id from the API
    """
    req = get_requester()
    return req.fetch_remote_id(id)


@shared_task
def fetch_remote_user(id: str, force_fetch: bool = False):
    """
    Fetch remote user from the API
    """
    req = get_requester()
    return req.fetch_remote_user(id, force_fetch)


@shared_task
def update_remote_user(id: str):
    """
    Update remote user from the API
    """
    u = User.objects.get(id=id)
    req = get_requester()
    req.fetch_remote_user(u.uri, force_fetch=True)


@shared_task
def fetch_remote_post_by_uri(uri: str, nested: int = 0):
    """
    Fetch remote post by uri from the API
    """
    req = get_requester()
    return req.fetch_remote_post_by_uri(uri, nested)


@shared_task
def fetch_remote_username(username: str, host: str):
    req = get_requester()
    return req.fetch_remote_username(username, host)


@shared_task
def send_follow_accept(follow_req_id: UUID) -> None:
    follow_req = UserFollowRequest.objects.get(id=follow_req_id)
    req = get_requester()
    req.send_follow_accept(follow_req)


@shared_task
def send_unfollow(follower_id: UUID, followee_id: UUID) -> None:
    follower = User.objects.get(id=follower_id)
    followee = User.objects.get(id=followee_id)
    req = get_requester()
    req.send_unfollow(follower, followee)


@shared_task
def send_follow_request(follow_req_id: UUID) -> None:
    follow_req = UserFollowRequest.objects.get(id=follow_req_id)
    req = get_requester()
    req.send_follow_request(follow_req)


@shared_task
def send_post_to_federated_servers(post_id: UUID) -> None:
    post = Post.objects.get(id=post_id)
    req = get_requester()
    req.send_post_to_federated_servers(post)


@shared_task
def gen_keypair_for_user(user_id: UUID) -> None:
    user = User.objects.get(id=user_id)
    keypair = generate_key_pair()
    user.private_key = keypair.private_key
    user.public_key = keypair.public_key
    user.save()
