use chrono::DateTime;
use chrono::Utc;
use derive_more::Constructor;
use getset::CopyGetters;
use getset::Getters;
use url::Url;

use crate::ServiceResult;
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

#[derive(Debug, Getters, CopyGetters, Serialize, Deserialize, Constructor)]
pub struct UserEntity {
    #[getset(get_copy = "pub")]
    id: UserID,
    #[getset(get = "pub")]
    username: Username,
    #[getset(get = "pub")]
    domain: Domain,
    #[getset(get = "pub")]
    nickname: Nickname,
    #[getset(get = "pub")]
    profile: UserProfile,
    #[getset(get = "pub")]
    config: UserConfig,

    #[getset(get_copy = "pub")]
    is_dirty: bool,
    #[getset(get_copy = "pub")]
    in_db: bool,
}

impl UserEntity {
    pub fn create_new(username: Username, nickname: Nickname) -> Self {
        Self {
            id: UserID::new_random(),
            username,
            domain: Domain::local(),
            nickname,
            profile: UserProfile::default(),
            config: UserConfig::default(),

            is_dirty: true,
            in_db: false,
        }
    }

    fn mark_dirty(&mut self) {
        self.is_dirty = true;
    }

    pub fn _set_saved(&mut self) {
        self.is_dirty = false;
        self.in_db = true;
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

#[derive(Debug, Getters, CopyGetters, Serialize, Deserialize, Constructor, Getters)]
pub struct UserProfile {
    #[getset(get = "pub")]
    bio: String,
    #[getset(get_copy = "pub")]
    avatar: Option<UploadID>,
    #[getset(get = "pub")]
    fetched_at: Option<DateTime<Utc>>,
    #[getset(get = "pub")]
    created_at: Option<DateTime<Utc>>,
}

impl Default for UserProfile {
    fn default() -> Self {
        Self {
            bio: String::new(),
            avatar: None,
            fetched_at: None,
            created_at: Some(Utc::now()),
        }
    }
}

#[derive(Debug, Serialize, Deserialize, Constructor, CopyGetters)]
pub struct UserConfig {
    #[getset(get_copy = "pub")]
    is_bot: bool,
    #[getset(get_copy = "pub")]
    is_admin: bool,
    #[getset(get_copy = "pub")]
    auto_follow_accept: bool,
    #[getset(get_copy = "pub")]
    hide_follows: bool,
}

impl Default for UserConfig {
    fn default() -> Self {
        Self {
            is_bot: false,
            is_admin: false,
            auto_follow_accept: true,
            hide_follows: false,
        }
    }
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

    pub fn local() -> Self {
        Domain(None)
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

#[derive(Debug, Getters, Constructor)]
pub struct UserAuthEntity {
    user_id: UserID,
    password_hash: HashedUserPassword,
}

impl UserAuthEntity {
    pub fn set_password_hash(&mut self, password_hash: HashedUserPassword) {
        self.password_hash = password_hash;
    }
}

#[derive(Debug, Clone)]
pub struct UserPassword {
    password: String,
}

impl UserPassword {
    pub fn new(password: String) -> Self {
        Self { password }
    }
}

#[derive(Debug, Clone)]
pub struct HashedUserPassword(String);

impl HashedUserPassword {
    pub fn from_str(s: impl Into<String>) -> Self {
        Self(s.into())
    }

    pub fn as_str(&self) -> &str {
        &self.0
    }

    pub fn into_inner(self) -> String {
        self.0
    }
}
