use serde::Serialize;

use crate::model::post::PostId;

use super::{user::UserId, DateTime, URI};

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
}

pub struct PostCommon {
    id: PostId,
    uri: Option<URI>, // when remote post
    author: UserId,
    privacy: PostPrivacy,
    created_at: DateTime,
    deleted_at: Option<DateTime>,
}

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

pub struct PostRepost {
    common: PostCommon,
    repost_of: PostRepostInfo,
}

pub struct PostReplyInfo {
    reply_to_id: PostId,
    reply_to_uri: Option<URI>, // when remote post
}

pub struct PostRepostInfo {
    reply_of_id: PostId,
    reply_of_uri: Option<URI>, // when remote post
}

// PostReaction value object
pub struct PostReaction {
    name: PostReactionName,
    count: i64,
}

// PostReactionName value object
pub struct PostReactionName(String);

// Post entity
pub enum Post {
    Normal(PostNonRepost),
    Repost(PostRepost),
}

impl Post {
    pub fn id(&self) -> &PostId {
        match self {
            Self::Normal(p) => &p.common.id,
            Self::Repost(p) => &p.common.id,
        }
    }
}

impl PartialEq for Post {
    fn eq(&self, other: &Self) -> bool {
        *self.id() == *other.id()
    }
}

impl Eq for Post {}
