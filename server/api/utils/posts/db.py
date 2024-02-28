import datetime
import uuid
from dataclasses import dataclass
from typing import Iterator

from django.db import transaction

from api import tasks
from api.models import Post, PostAttachment, PostHashtag, PostMention, User
from api.utils.posts.content import find_hashtags, find_mentions
from api.utils.posts.privacy import PostPrivacy


@dataclass
class CreatePostData:
    poster: User
    privacy: PostPrivacy
    content: str | None  # None if repost
    repost_of_id: uuid.UUID | None = None
    reply_to_id: uuid.UUID | None = None
    attached_uploads: list[uuid.UUID] | None = None
    uri: str | None = None  # None if posted by local user
    created_at: datetime.datetime | None = None


def create_post(data: CreatePostData) -> Post:
    poster = data.poster

    # begins transaction
    with transaction.atomic():
        post = Post.objects.create(
            poster=poster,
            content=data.content,
            reply_to_id=data.reply_to_id,
            repost_of_id=data.repost_of_id,
            privacy=data.privacy.value,
            uri=data.uri,
            created_at=(
                data.created_at
                if data.created_at is not None
                else datetime.datetime.now(datetime.timezone.utc)
            ),
        )

        # region hashtag
        # find hashtags
        if data.content is not None:
            hashtags = find_hashtags(data.content)
        else:
            # fetch hashtags of the original post
            repost_of = Post.objects.get(id=data.repost_of_id)
            hashtags = PostHashtag.objects.filter(post=repost_of).values_list(
                "hashtag", flat=True
            )

        for hashtag in hashtags:
            PostHashtag.objects.create(post=post, hashtag=hashtag)
        # endregion

        # region mentions
        # find mentions
        if data.content is not None:
            raw_mentions = find_mentions(data.content)
            mentions = []
            for raw_mention in raw_mentions:
                if (user := raw_mention.to_user_spec().get_user_model()) is not None:
                    mentions.append(user)
        else:
            # find mentions of the original post
            # TODO: use repost_of fetched above
            repost_of = Post.objects.get(id=data.repost_of_id)
            mention_models: Iterator[PostMention] = repost_of.mentions.all()
            mentions = []
            for mention_model in mention_models:
                mentions.append(mention_model.target_user)

        for mention in mentions:
            # TODO: bulk insert
            PostMention.objects.create(
                post=post,
                target_user=mention,
            )
        # endregion

        # process attached uploads
        if data.attached_uploads:
            for uploaded_file in data.attached_uploads:
                PostAttachment.objects.create(post=post, file=uploaded_file)

    # deliver to other instances if poster is a local user
    if poster.host is None:
        tasks.send_post_to_federated_servers.delay(post.id)

    return post
