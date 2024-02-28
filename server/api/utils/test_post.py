import pytest

from . import post
from .post import MentionTarget


def test_find_hashtag():
    f = post.find_hashtags
    assert f("Here is a post with a #hashtag") == ["hashtag"]
    assert f("This post contains multiple #hashtags, with #different #tags.") == [
        "hashtags",
        "different",
        "tags",
    ]
    assert f("This is a post without any hashtags.") == []
    assert f("Hashtags can have numbers like #tag1 and #2tag") == ["tag1", "2tag"]
    assert f("Hashtags with non-Latin characters #тег #标签") == ["тег", "标签"]
    assert f("Some posts repeat hashtags #tag #other #tag") == ["tag", "other"]
    assert f("Hashtag at the end #end") == ["end"]
    assert f("Hashtags followed by punctuation #tag!") == ["tag"]
    assert f("String with just #") == []


def test_find_mentions():
    f = post.find_mentions
    t = MentionTarget
    assert f("Hello @user") == [t("user", None)]
    assert f("Hello @user@example.com") == [t("user", "example.com")]
    assert f(
        "Hello @user and @other@example.com, and @another@another.example.com"
    ) == [
        t("user", None),
        t("other", "example.com"),
        t("another", "another.example.com"),
    ]

    # No mentions
    assert f("Hello there") == []

    # Multiple mentions without domain
    assert f("Hello @user1 and @user2") == [t("user1", None), t("user2", None)]

    # Mixed mentions
    assert f("Hello @user, @other@example.com, and just text") == [
        t("user", None),
        t("other", "example.com"),
    ]

    # Mentions with special characters
    assert f("@user_name and @user-name@example-domain.com") == [
        t("user_name", None),
        t("user-name", "example-domain.com"),
    ]

    # Mentions close to punctuation
    assert f("Hello @user, how are you? @another_user!") == [
        t("user", None),
        t("another_user", None),
    ]

    # Mentions in complex strings
    assert f("@user: Check this out! @another_user@example.com, isn't it cool?") == [
        t("user", None),
        t("another_user", "example.com"),
    ]

    # Uppercase mentions
    assert f("Hello @User and @OtherUser") == [t("User", None), t("OtherUser", None)]

    # Mentions in multiline string
    assert f("Hello @user\nHow are you @other_user?") == [
        t("user", None),
        t("other_user", None),
    ]
