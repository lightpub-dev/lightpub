use derive_more::Constructor;
use serde::Serialize;
use uuid::Uuid;

use super::{user::UserId, DateTime, URI};

// PostId value object
#[derive(Debug, PartialEq, Eq, Clone, Copy)]
pub struct PostId(Uuid);

impl PostId {
    pub fn from_uuid(uuid: Uuid) -> Self {
        Self(uuid)
    }

    pub fn from_str(uuid: &str) -> Option<Self> {
        Uuid::parse_str(uuid).map(Self).ok()
    }

    pub fn id(&self) -> Uuid {
        self.0
    }

    pub fn to_string(&self) -> String {
        self.0.simple().to_string()
    }
}

// PostPrivacy value object
#[derive(Debug, Clone, PartialEq, Eq, Copy, Serialize)]
pub enum PostPrivacy {
    #[serde(rename = "public")]
    Public,
    #[serde(rename = "unlisted")]
    Unlisted,
    #[serde(rename = "follower")]
    Followers,
    #[serde(rename = "private")]
    Private,
}

// PostContent value object
#[derive(Debug, Clone, Serialize)]
pub struct PostContent(String);

impl PostContent {
    pub fn from_string(content: impl Into<String>) -> Self {
        Self(content.into())
    }

    pub fn to_str(&self) -> &str {
        &self.0
    }
}

#[derive(Debug, Constructor)]
pub struct PostCommon {
    id: PostId,
    uri: Option<URI>, // when remote post
    author: UserId,
    privacy: PostPrivacy,
    created_at: DateTime,
    deleted_at: Option<DateTime>,
}

#[derive(Debug, Constructor)]
pub struct PostNonRepost {
    common: PostCommon,
    content: PostContent,
    reply_to: Option<PostReplyInfo>,
    mentioned_users: Vec<UserId>,

    reply_count: i64,
    repost_count: i64,
    quote_count: i64,
    reactions: Vec<PostReaction>,
}

#[derive(Debug, Constructor)]
pub struct PostQuote {
    common: PostCommon,
    content: PostContent,
    quote_of: PostQuoteInfo,
    mentioned_users: Vec<UserId>,

    reply_count: i64,
    repost_count: i64,
    quote_count: i64,
    reactions: Vec<PostReaction>,
}

#[derive(Debug, Constructor)]
pub struct PostRepost {
    common: PostCommon,
    repost_of: PostRepostInfo,
}

#[derive(Debug, Constructor)]
pub struct PostReplyInfo {
    reply_to_id: PostId,
    // reply_to_uri: Option<URI>, // when remote post
}

#[derive(Debug, Constructor)]
pub struct PostRepostInfo {
    reply_of_id: PostId,
    // reply_of_uri: Option<URI>, // when remote post
}

#[derive(Debug, Constructor)]
pub struct PostQuoteInfo {
    quote_of_id: PostId,
    // quote_of_uri: Option<URI>, // when remote post
}

#[derive(Debug, Constructor, Clone, PartialEq, Eq)]
// PostReaction value object
pub struct PostReaction {
    name: PostReactionName,
    count: i64,
}

// PostReactionName value object
#[derive(Debug, Clone, PartialEq, Eq)]
pub struct PostReactionName(String);

impl PostReactionName {
    pub fn from_string(name: impl Into<String>) -> Self {
        Self(name.into())
    }

    pub fn new(name: impl Into<String>) -> Self {
        Self::from_string(name)
    }

    pub fn to_str(&self) -> &str {
        &self.0
    }
}

// Post entity
pub enum Post {
    Normal(PostNonRepost),
    Repost(PostRepost),
    Quote(PostQuote),
}

impl Post {
    fn common(&self) -> &PostCommon {
        match self {
            Self::Normal(p) => &p.common,
            Self::Repost(p) => &p.common,
            Self::Quote(q) => &q.common,
        }
    }

    pub fn id(&self) -> PostId {
        self.common().id
    }

    pub fn author_id(&self) -> UserId {
        self.common().author
    }

    pub fn uri(&self) -> Option<&URI> {
        self.common().uri.as_ref()
    }

    pub fn privacy(&self) -> PostPrivacy {
        self.common().privacy
    }

    pub fn content(&self) -> Option<&PostContent> {
        match self {
            Self::Normal(p) => Some(&p.content),
            Self::Quote(q) => Some(&q.content),
            Self::Repost(_) => None,
        }
    }

    pub fn reply_to(&self) -> Option<&PostReplyInfo> {
        match self {
            Self::Normal(p) => p.reply_to.as_ref(),
            Self::Quote(_) => None,
            Self::Repost(_) => None,
        }
    }

    pub fn repost_of(&self) -> Option<&PostRepostInfo> {
        match self {
            Self::Normal(_) => None,
            Self::Quote(_) => None,
            Self::Repost(p) => Some(&p.repost_of),
        }
    }

    pub fn quote_of(&self) -> Option<&PostQuoteInfo> {
        match self {
            Self::Normal(_) => None,
            Self::Quote(q) => Some(&q.quote_of),
            Self::Repost(_) => None,
        }
    }

    pub fn created_at(&self) -> &DateTime {
        &self.common().created_at
    }
}

impl PartialEq for Post {
    fn eq(&self, other: &Self) -> bool {
        self.id() == other.id()
    }
}

impl Eq for Post {}

impl PostReplyInfo {
    pub fn id(&self) -> PostId {
        self.reply_to_id
    }
}

impl PostRepostInfo {
    pub fn id(&self) -> PostId {
        self.reply_of_id
    }
}
