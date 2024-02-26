import uuid

from django.db import models


# Create your models here.
class User(models.Model):
    id = models.UUIDField(primary_key=True, default=uuid.uuid4)
    username = models.CharField(
        max_length=64,
    )
    host = models.CharField(max_length=128, blank=True, null=True, default=None)
    bpassword = models.CharField(max_length=60, blank=True, null=True)
    nickname = models.CharField(max_length=255)
    bio = models.TextField(default="")
    avatar = models.ForeignKey(
        "UploadedFile",
        on_delete=models.SET_NULL,
        related_name="avatar_for",
        blank=True,
        null=True,
    )
    uri = models.CharField(max_length=512, null=True, blank=True)
    inbox = models.CharField(max_length=512, null=True, blank=True)
    outbox = models.CharField(max_length=512, null=True, blank=True)
    private_key = models.TextField(null=True, blank=True)
    public_key = models.TextField(null=True, blank=True)
    created_at = models.DateTimeField(auto_now_add=True)
    deleted_at = models.DateTimeField(null=True, blank=True)

    def __str__(self) -> str:
        s = f"@{self.username}"
        if self.host:
            s += f"@{self.host}"

        s += f" ({self.id})"
        return s


class RemoteUserInfo(models.Model):
    user = models.OneToOneField(
        User,
        on_delete=models.CASCADE,
        primary_key=True,
        related_name="remote_user_info",
    )
    following = models.CharField(max_length=512, null=True, blank=True)
    followers = models.CharField(max_length=512, null=True, blank=True)
    liked = models.CharField(max_length=512, null=True, blank=True)
    preferred_username = models.CharField(max_length=128, null=True, blank=True)

    last_fetched_at = models.DateTimeField(auto_now=True)


class PublicKey(models.Model):
    id = models.AutoField(primary_key=True)
    uri = models.CharField(max_length=512, null=False, blank=False)
    user = models.ForeignKey(
        User,
        on_delete=models.CASCADE,
        related_name="public_keys",
    )
    public_key_pem = models.TextField(blank=False, null=True)
    last_fetched_at = models.DateTimeField(auto_now=True)


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


class UserFollowRequest(models.Model):
    id = models.UUIDField(primary_key=True, default=uuid.uuid4)
    uri = models.CharField(max_length=512, null=True, blank=True, unique=True)
    follower = models.ForeignKey(
        User, on_delete=models.CASCADE, related_name="follow_requests"
    )
    followee = models.ForeignKey(
        User, on_delete=models.CASCADE, related_name="follower_requests"
    )
    incoming = models.BooleanField(default=False)
    created_at = models.DateTimeField(auto_now_add=True)


class UserToken(models.Model):
    id = models.AutoField(primary_key=True)
    user = models.ForeignKey(User, on_delete=models.CASCADE)
    token = models.CharField(max_length=64, unique=True)
    created_at = models.DateTimeField(auto_now_add=True)
    last_used_at = models.DateTimeField(auto_now=True)


class Post(models.Model):
    id = models.UUIDField(primary_key=True, default=uuid.uuid4)
    uri = models.CharField(max_length=512, null=True, blank=True)  # remote post only
    poster = models.ForeignKey(User, on_delete=models.CASCADE, related_name="posts")
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
    # to get the full post, use "uri" to fetch from the remote server
    partial = models.BooleanField(default=False)

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
    user = models.ForeignKey(User, on_delete=models.CASCADE, related_name="favorites")
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
    user = models.ForeignKey(User, on_delete=models.CASCADE, related_name="bookmarks")
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


class UploadedFile(models.Model):
    id = models.UUIDField(primary_key=True, default=uuid.uuid4)
    uploader = models.ForeignKey(
        User, on_delete=models.CASCADE, related_name="uploaded_files"
    )
    media_type = models.CharField(max_length=255, default="")
    file = models.FileField(upload_to="uploads/")
    created_at = models.DateTimeField(auto_now_add=True)


class PostAttachment(models.Model):
    id = models.UUIDField(primary_key=True, default=uuid.uuid4)
    post = models.ForeignKey(Post, on_delete=models.CASCADE, related_name="attachments")
    file = models.ForeignKey(
        UploadedFile,
        on_delete=models.SET_NULL,
        related_name="attachments",
        blank=False,
        null=True,
    )


class PostReaction(models.Model):
    id = models.AutoField(primary_key=True)
    post = models.ForeignKey(Post, on_delete=models.CASCADE, related_name="reactions")
    user = models.ForeignKey(User, on_delete=models.CASCADE)
    emoji = models.CharField(max_length=255)
    created_at = models.DateTimeField(auto_now_add=True)

    class Meta:
        constraints = [
            models.UniqueConstraint(
                fields=["post", "user", "emoji"],
                name="unique_post_reaction",
            )
        ]


class FederatedServer(models.Model):
    id = models.AutoField(primary_key=True)
    host = models.CharField(max_length=128, unique=True)
    inserted_at = models.DateTimeField(auto_now_add=True)
