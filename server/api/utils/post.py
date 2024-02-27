import re
from dataclasses import dataclass


def find_hashtags(content: str) -> list[str]:
    hashtags = []
    in_hashtag = False
    hashtag_start_index = 0
    for i, ch in enumerate(content):
        if ch == "#":
            if not in_hashtag:
                in_hashtag = True
                hashtag_start_index = i
            else:
                # Reset if another # is found immediately after
                in_hashtag = False
        elif not ch.isalnum() and ch not in ["_", "-"]:
            # Word boundary detected
            if in_hashtag:
                hashtag = content[hashtag_start_index:i]
                if hashtag != "#":  # Ignore single '#' entries
                    hashtags.append(hashtag[1:])
                in_hashtag = False
        # Note: The Go code snippet handles end-of-content logic implicitly
    # Check if the content ends with a hashtag
    if in_hashtag:
        hashtag = content[hashtag_start_index:]
        if hashtag != "#":
            hashtags.append(hashtag[1:])

    # Remove duplicates while preserving order
    unique_hashtags = []
    for hashtag in hashtags:
        if hashtag not in unique_hashtags:
            unique_hashtags.append(hashtag)
    return unique_hashtags


@dataclass
class MentionTarget:
    username: str
    host: str | None


_mention_re = re.compile(r"@([a-zA-Z0-9_\-]+)(?:@([a-zA-Z0-9_\-\.]+))?")


def find_mentions(content: str) -> list[MentionTarget]:
    all_matches = _mention_re.findall(content)

    mentions = []
    for match in all_matches:
        username = match[0]
        host = match[1] if match[1] else None
        mentions.append(MentionTarget(username, host))

    return mentions
