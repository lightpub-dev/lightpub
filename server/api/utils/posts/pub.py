from api.models import Post
from api.utils.get_id import get_post_id, get_user_id
from api.utils.inbox import calculate_to_and_cc


def create_post_object(post: Post) -> dict:
    """
    Create a post object from a Post model instance.
    """
    tocc = calculate_to_and_cc(post)

    post_id = get_post_id(post)
    sender_id = get_user_id(post.poster)
    post_object = {
        "id": post_id,
        "attributedTo": sender_id,
        "type": "Note",
        "to": tocc["to"],
        "cc": tocc["cc"],
        "content": post.content,
        "published": post.created_at.isoformat(),
        "sensitive": False,
    }
    if post.reply_to:
        post_object["inReplyTo"] = get_post_id(post.reply_to, use_remote_uri=True)
    return post_object
