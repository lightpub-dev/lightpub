use lazy_static::lazy_static;
use regex::Regex;
use uuid::Uuid;

use lightpub_model::UserSpecifier;

#[derive(Debug, Clone)]
pub enum PostSpecifier {
    ID(Uuid),
    URI(String),
}

impl PostSpecifier {
    pub fn from_id(id: impl Into<Uuid>) -> Self {
        PostSpecifier::ID(id.into())
    }

    pub fn from_uri(uri: impl Into<String>) -> Self {
        PostSpecifier::URI(uri.into())
    }
}

impl From<Uuid> for PostSpecifier {
    fn from(id: Uuid) -> Self {
        PostSpecifier::ID(id)
    }
}

pub fn find_hashtags(content: &str) -> Vec<String> {
    let mut hashtags: Vec<String> = Vec::new();
    let mut in_hashtag = false;
    let mut hashtag_start_index: usize = 0;
    let chars: Vec<char> = content.chars().collect();
    let mut positions: Vec<usize> = vec![]; // To store the start positions of hashtags

    for (i, &ch) in chars.iter().enumerate() {
        if ch == '#' {
            if !in_hashtag {
                in_hashtag = true;
                hashtag_start_index = i;
            } else {
                // Reset if another # is found immediately after
                in_hashtag = false;
            }
        } else if !ch.is_alphanumeric() && ch != '_' && ch != '-' {
            // Word boundary detected
            if in_hashtag {
                positions.push(hashtag_start_index);
                in_hashtag = false;
            }
        }
    }
    // Check if the content ends with a hashtag
    if in_hashtag {
        positions.push(hashtag_start_index);
    }

    for &pos in &positions {
        let mut hashtag = chars[pos + 1..].iter().collect::<String>();
        if let Some(non_alnum_pos) =
            hashtag.find(|c: char| !c.is_alphanumeric() && c != '_' && c != '-')
        {
            hashtag.truncate(non_alnum_pos);
        }
        if !hashtag.is_empty() && !hashtags.contains(&hashtag) {
            hashtags.push(hashtag);
        }
    }

    hashtags
}

pub type MentionTarget = UserSpecifier;

pub fn find_mentions(content: &str) -> Vec<MentionTarget> {
    lazy_static! {
        static ref MENTION_RE: Regex =
            Regex::new(r"@([a-zA-Z0-9_\-\.]+)(?:@([a-zA-Z0-9_\-\.]+))?").unwrap();
    }
    let mut mentions = Vec::new();

    for cap in MENTION_RE.captures_iter(content) {
        let username = cap[1].to_string();
        let host = if let Some(matched_host) = cap.get(2) {
            Some(matched_host.as_str().to_string())
        } else {
            None
        };
        mentions.push(UserSpecifier::from_username(username, host));
    }

    mentions
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn test_find_hashtag_basic() {
        assert_eq!(
            find_hashtags("Here is a post with a #hashtag"),
            vec!["hashtag".to_string()]
        );
    }

    #[test]
    fn test_find_hashtag_multiple() {
        assert_eq!(
            find_hashtags("This post contains multiple #hashtags, with #different #tags."),
            vec![
                "hashtags".to_string(),
                "different".to_string(),
                "tags".to_string()
            ]
        );
    }

    #[test]
    fn test_find_hashtag_none() {
        assert_eq!(
            find_hashtags("This is a post without any hashtags."),
            Vec::<String>::new()
        );
    }

    #[test]
    fn test_find_hashtag_with_numbers() {
        assert_eq!(
            find_hashtags("Hashtags can have numbers like #tag1 and #2tag"),
            vec!["tag1".to_string(), "2tag".to_string()]
        );
    }

    #[test]
    fn test_find_hashtag_non_latin() {
        assert_eq!(
            find_hashtags("Hashtags with non-Latin characters #тег #标签"),
            vec!["тег".to_string(), "标签".to_string()]
        );
    }

    #[test]
    fn test_find_hashtag_repeats() {
        assert_eq!(
            find_hashtags("Some posts repeat hashtags #tag #other #tag"),
            vec!["tag".to_string(), "other".to_string()]
        );
    }

    #[test]
    fn test_find_hashtag_end() {
        assert_eq!(
            find_hashtags("Hashtag at the end #end"),
            vec!["end".to_string()]
        );
    }

    #[test]
    fn test_find_hashtag_followed_by_punctuation() {
        assert_eq!(
            find_hashtags("Hashtags followed by punctuation #tag!"),
            vec!["tag".to_string()]
        );
    }

    #[test]
    fn test_find_hashtag_just_hash() {
        assert_eq!(find_hashtags("String with just #"), Vec::<String>::new());
    }

    #[test]
    fn test_find_mentions() {
        let f = |s: &str| find_mentions(s);

        // Helper function to easily create MentionTarget instances
        let t = |username: &str, host: Option<&str>| {
            UserSpecifier::from_username(username.to_string(), host.map(|h| h.to_string()))
        };

        assert_eq!(f("Hello @user"), vec![t("user", None)]);
        assert_eq!(
            f("Hello @user@example.com"),
            vec![t("user", Some("example.com"))]
        );
        assert_eq!(
            f("Hello @user and @other@example.com, and @another@another.example.com"),
            vec![
                t("user", None),
                t("other", Some("example.com")),
                t("another", Some("another.example.com")),
            ]
        );
        assert_eq!(
            f("@alex.7552441788648396124@tester.tinax.local hi"),
            vec![t("alex.7552441788648396124", Some("tester.tinax.local"))]
        );

        // No mentions
        assert_eq!(f("Hello there"), vec![]);

        // Multiple mentions without domain
        assert_eq!(
            f("Hello @user1 and @user2"),
            vec![t("user1", None), t("user2", None)]
        );

        // Mixed mentions
        assert_eq!(
            f("Hello @user, @other@example.com, and just text"),
            vec![t("user", None), t("other", Some("example.com")),]
        );

        // Mentions with special characters
        assert_eq!(
            f("@user_name and @user-name@example-domain.com"),
            vec![
                t("user_name", None),
                t("user-name", Some("example-domain.com")),
            ]
        );

        // Mentions close to punctuation
        assert_eq!(
            f("Hello @user, how are you? @another_user!"),
            vec![t("user", None), t("another_user", None),]
        );

        // Mentions in complex strings
        assert_eq!(
            f("@user: Check this out! @another_user@example.com, isn't it cool?"),
            vec![t("user", None), t("another_user", Some("example.com")),]
        );

        // Uppercase mentions
        assert_eq!(
            f("Hello @User and @OtherUser"),
            vec![t("User", None), t("OtherUser", None)]
        );

        // Mentions in multiline string
        assert_eq!(
            f("Hello @user\nHow are you @other_user?"),
            vec![t("user", None), t("other_user", None),]
        );
    }
}
