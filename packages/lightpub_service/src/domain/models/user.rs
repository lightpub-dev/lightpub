use chrono::DateTime;
use chrono::Utc;
use derive_getters::Getters;
use derive_more::Constructor;
use url::Url;

use crate::domain::models::apub::ActorID;
use crate::domain::models::apub::ApubPrivateKey;
use crate::domain::models::apub::ApubPublicKey;
use crate::domain::models::upload::UploadID;

use std::str::FromStr;

use super::IdParseError;
use super::slice_to_bytes;
use derive_more::Display;
use serde::{Deserialize, Serialize};
use ulid::Ulid;

#[derive(Debug, Clone, Serialize, Deserialize, PartialEq, Eq, Display, Hash, Copy)]
pub struct UserID(Ulid);

impl FromStr for UserID {
    type Err = IdParseError;
    fn from_str(s: &str) -> Result<Self, Self::Err> {
        Ulid::from_str(s)
            .map(UserID)
            .map_err(|_| IdParseError::ParseError)
    }
}

impl UserID {
    pub fn new_random() -> Self {
        Self(Ulid::new())
    }

    pub fn as_db(&self) -> Vec<u8> {
        self.0.to_bytes().to_vec()
    }
    pub fn from_db_trusted(db: Vec<u8>) -> Self {
        // check fi
        Self(Ulid::from_bytes(slice_to_bytes(&db)))
    }
}

#[derive(Debug, Getters, Serialize, Deserialize, Constructor)]
pub struct UserEntity {
    id: UserID,
    username: Username,
    domain: Domain,
    nickname: Nickname,
    profile: UserProfile,
    config: UserConfig,

    is_dirty: bool,
    in_db: bool,
}

impl UserEntity {
    fn mark_dirty(&mut self) {
        self.is_dirty = true;
    }

    pub fn set_nickname(&mut self, nickname: Nickname) {
        self.nickname = nickname;
        self.mark_dirty();
    }

    pub fn set_bio(&mut self, bio: String) {
        self.profile.bio = bio;
        self.mark_dirty();
    }

    pub fn set_avatar(&mut self, avatar: Option<UploadID>) {
        self.profile.avatar = avatar;
        self.mark_dirty();
    }

    pub fn set_is_bot(&mut self, is_bot: bool) {
        self.config.is_bot = is_bot;
        self.mark_dirty();
    }

    pub fn set_is_admin(&mut self, is_admin: bool) {
        self.config.is_admin = is_admin;
        self.mark_dirty();
    }

    pub fn set_auto_follow_accept(&mut self, auto_follow_accept: bool) {
        self.config.auto_follow_accept = auto_follow_accept;
        self.mark_dirty();
    }

    pub fn set_hide_follows(&mut self, hide_follows: bool) {
        self.config.hide_follows = hide_follows;
        self.mark_dirty();
    }
}

#[derive(Debug, Getters, Serialize, Deserialize, Constructor)]
pub struct UserProfile {
    bio: String,
    avatar: Option<UploadID>,
    fetched_at: Option<DateTime<Utc>>,
    created_at: Option<DateTime<Utc>>,
}

#[derive(Debug, Getters, Serialize, Deserialize, Constructor)]
pub struct UserConfig {
    is_bot: bool,
    is_admin: bool,
    auto_follow_accept: bool,
    hide_follows: bool,
}

#[derive(Debug, Getters)]
pub struct ApubActorEntity {
    user_id: UserID,
    actor_id: ActorID,
    url: Url,
    view_url: Option<Url>,
    inbox: Url,
    outbox: Url,
    shared_inbox: Option<Url>,
    followers: Option<Url>,
    following: Option<Url>,
    private_key: Option<ApubPrivateKey>,
    public_key: ApubPublicKey,
}

#[derive(Debug, Clone, Serialize, Deserialize, PartialEq, Eq, Display, Hash)]
pub struct Domain(Option<String>);

impl Domain {
    pub fn from_str(s: impl Into<String>) -> Self {
        let s = s.into();
        if s == "" {
            Domain(None)
        } else {
            Domain(Some(s))
        }
    }

    pub fn is_local(&self) -> bool {
        self.0.is_none()
    }

    pub fn as_str(&self) -> Option<&str> {
        self.0.as_deref()
    }

    pub fn as_db(&self) -> &str {
        match &self.0 {
            Some(d) => d,
            None => "",
        }
    }
}

#[derive(Debug, Clone, Serialize, Deserialize, PartialEq, Eq, Display, Hash)]
pub struct Username(String);

impl Username {
    pub fn from_str(s: impl Into<String>) -> Self {
        Self(s.into())
    }

    pub fn as_str(&self) -> &str {
        &self.0
    }
}

#[derive(Debug, Clone, Serialize, Deserialize, PartialEq, Eq, Display, Hash)]
pub struct Nickname(String);

impl Nickname {
    pub fn from_str(s: impl Into<String>) -> Self {
        Self(s.into())
    }

    pub fn as_str(&self) -> &str {
        &self.0
    }
}
