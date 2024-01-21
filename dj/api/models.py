import uuid

from django.db import models


# Create your models here.
class User(models.Model):
    id = models.UUIDField(primary_key=True, default=uuid.uuid4)
    username = models.CharField(max_length=64, unique=True)
    host = models.CharField(max_length=128)
    bpassword = models.CharField(max_length=60)
    nickname = models.CharField(max_length=255)
    bio = models.TextField(default="")
    url = models.CharField(max_length=512, null=True, blank=True)
    inbox = models.CharField(max_length=512, null=True, blank=True)
    outbox = models.CharField(max_length=512, null=True, blank=True)
    created_at = models.DateTimeField(auto_now_add=True)
    deleted_at = models.DateTimeField(null=True, blank=True)


class UserProfileLabel(models.Model):
    id = models.AutoField(primary_key=True)
    user = models.ForeignKey(User, on_delete=models.CASCADE, related_name="labels")
    order = models.IntegerField()
    key = models.CharField(max_length=1000)
    value = models.CharField(max_length=1000)

    class Meta:
        constraints = [
            models.UniqueConstraint(
                fields=["user", "order"],
                name="unique_user_profile_label_order",
            )
        ]


class UserFollow(models.Model):
    id = models.AutoField(primary_key=True)
    follower = models.ForeignKey(
        User, on_delete=models.CASCADE, related_name="followings"
    )
    followee = models.ForeignKey(
        User, on_delete=models.CASCADE, related_name="followers"
    )
    created_at = models.DateTimeField(auto_now_add=True)

    class Meta:
        constraints = [
            models.UniqueConstraint(
                fields=["follower", "followee"],
                name="unique_user_follow",
            )
        ]


class UserToken(models.Model):
    id = models.AutoField(primary_key=True)
    user = models.ForeignKey(User, on_delete=models.CASCADE)
    token = models.CharField(max_length=64, unique=True)
    created_at = models.DateTimeField(auto_now_add=True)
    last_used_at = models.DateTimeField(auto_now=True)


class Post(models.Model):
    id = models.UUIDField(primary_key=True, default=uuid.uuid4)
    poster = models.ForeignKey(User, on_delete=models.CASCADE)
    content = models.TextField(
        max_length=10000, null=True, blank=True
    )  # NULL for repost
    inserted_at = models.DateTimeField(auto_now_add=True)
    created_at = models.DateTimeField(auto_now_add=True)
    privacy = models.SmallIntegerField(
        choices=[(0, "public"), (1, "unlisted"), (2, "follower"), (3, "private")]
    )
    reply_to = models.ForeignKey(
        "self", on_delete=models.CASCADE, null=True, blank=True, related_name="replies"
    )
    repost_of = models.ForeignKey(
        "self", on_delete=models.CASCADE, null=True, blank=True, related_name="reposts"
    )
    edited = models.BooleanField(default=False)
    deleted_at = models.DateTimeField(null=True, blank=True, default=None)


class PostHashtag(models.Model):
    id = models.AutoField(primary_key=True)
    post = models.ForeignKey(Post, on_delete=models.CASCADE, related_name="hashtags")
    hashtag = models.CharField(max_length=255)

    class Meta:
        constraints = [
            models.UniqueConstraint(
                fields=["post", "hashtag"],
                name="unique_post_hashtag",
            )
        ]


class PostFavorite(models.Model):
    id = models.AutoField(primary_key=True)
    post = models.ForeignKey(Post, on_delete=models.CASCADE, related_name="favorites")
    user = models.ForeignKey(User, on_delete=models.CASCADE)
    created_at = models.DateTimeField(auto_now_add=True)

    class Meta:
        constraints = [
            models.UniqueConstraint(
                fields=["post", "user"],
                name="unique_post_favorite",
            )
        ]


class PostBookmark(models.Model):
    id = models.AutoField(primary_key=True)
    post = models.ForeignKey(Post, on_delete=models.CASCADE, related_name="bookmarks")
    user = models.ForeignKey(User, on_delete=models.CASCADE)
    created_at = models.DateTimeField(auto_now_add=True)

    class Meta:
        constraints = [
            models.UniqueConstraint(
                fields=["post", "user"],
                name="unique_post_bookmark",
            )
        ]


class PostMention(models.Model):
    id = models.AutoField(primary_key=True)
    post = models.ForeignKey(Post, on_delete=models.CASCADE, related_name="mentions")
    target_user = models.ForeignKey(User, on_delete=models.CASCADE)
    created_at = models.DateTimeField(auto_now_add=True)

    class Meta:
        constraints = [
            models.UniqueConstraint(
                fields=["post", "target_user"],
                name="unique_post_mention",
            )
        ]
