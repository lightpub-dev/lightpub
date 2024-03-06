use derive_builder::Builder;
use derive_getters::Getters;
use rsa::RsaPrivateKey;
use serde::{ser::SerializeStruct, Deserialize, Serialize};
use uuid::fmt::Simple;

#[derive(Debug)]
pub struct User {
    pub id: sqlx::types::uuid::fmt::Simple,
    pub username: String,
    pub host: Option<String>,
    // pub bpasswd: Option<String>, // intentionally omitted to prevent accidental password leaks
    pub nickname: String,
    pub bio: String,
    pub uri: Option<String>,
    pub shared_inbox: Option<String>,
    pub inbox: Option<String>,
    pub outbox: Option<String>,
    // pub private_key: Option<String>, // intentionally omitted to prevent accidental private key leaks
    pub public_key: Option<String>,
    pub created_at: chrono::NaiveDateTime,
}

#[derive(Debug, Clone, PartialEq, Eq, Copy)]
pub enum PostPrivacy {
    Public,
    Unlisted,
    Followers,
    Private,
}

pub trait HasRemoteUri {
    fn get_local_id(&self) -> String;
    fn get_remote_uri(&self) -> Option<String>;
}

pub trait ApubRenderablePost {
    type Poster: HasRemoteUri;
    fn id(&self) -> Simple;
    fn uri(&self) -> Option<String>;
    fn content(&self) -> Option<String>;
    fn poster(&self) -> Self::Poster;
    fn privacy(&self) -> PostPrivacy;
    fn created_at(&self) -> chrono::DateTime<chrono::Utc>;
}

pub trait ApubRenderableUser {
    fn id(&self) -> Simple;
    fn uri(&self) -> Option<String>;
    fn username(&self) -> String;
    fn nickname(&self) -> String;
    // fn bio(&self) -> String;
    fn public_key(&self) -> Option<String>;
    fn created_at(&self) -> chrono::DateTime<chrono::Utc>;
}

impl<'de> Deserialize<'de> for PostPrivacy {
    fn deserialize<D>(deserializer: D) -> Result<Self, D::Error>
    where
        D: serde::Deserializer<'de>,
    {
        let s = String::deserialize(deserializer)?;
        use PostPrivacy::*;
        match s.as_str() {
            "public" => Ok(Public),
            "unlisted" => Ok(Unlisted),
            "followers" => Ok(Followers),
            "private" => Ok(Private),
            _ => Err(serde::de::Error::custom("invalid privacy value")),
        }
    }
}

impl PostPrivacy {
    pub fn to_db(&self) -> String {
        use PostPrivacy::*;
        match self {
            Public => "public",
            Unlisted => "unlisted",
            Followers => "followers",
            Private => "private",
        }
        .to_string()
    }
}

#[derive(Debug, Builder, Getters)]
pub struct ApubWebfingerResponse {
    api_url: String,
    profile_url: Option<String>,
}

#[derive(Debug)]
pub enum ApubActivity {}

impl ApubActivity {
    pub fn to_json(&self) -> String {
        todo!()
    }
}

#[derive(Debug, Builder)]
pub struct ApubNote {
    id: String,
    attributed_to: String,
    to: Vec<String>,
    cc: Vec<String>,
    content: String,
    published: chrono::DateTime<chrono::Utc>,
    sensitive: bool,
}

impl Serialize for ApubNote {
    fn serialize<S>(&self, serializer: S) -> Result<S::Ok, S::Error>
    where
        S: serde::Serializer,
    {
        let mut state = serializer.serialize_struct("ApubNote", 7)?;
        state.serialize_field("id", &self.id)?;
        state.serialize_field("attributedTo", &self.attributed_to)?;
        state.serialize_field("to", &self.to)?;
        state.serialize_field("cc", &self.cc)?;
        state.serialize_field("content", &self.content)?;
        state.serialize_field("published", &self.published)?;
        state.serialize_field("sensitive", &self.sensitive)?;
        state.serialize_field("type", "Note")?;
        state.end()
    }
}

#[derive(Debug, Deserialize, Builder, Clone)]
#[serde(rename_all = "camelCase")]
pub struct ApubPerson {
    id: String,
    name: Option<String>,
    inbox: String,
    outbox: String,
    following: Option<String>,
    followers: Option<String>,
    liked: Option<String>,
    preferred_username: Option<String>,
    shared_inbox: Option<String>,
    public_key: Option<ApubPublicKey>,
}

#[derive(Debug, Deserialize, Builder, Clone)]
#[serde(rename_all = "camelCase")]
pub struct ApubPublicKey {
    id: String,
    owner: String,
    public_key_pem: String,
}

impl Serialize for ApubPublicKey {
    fn serialize<S>(&self, serializer: S) -> Result<S::Ok, S::Error>
    where
        S: serde::Serializer,
    {
        let mut s = serializer.serialize_struct("ApubPublicKey", 3)?;
        s.serialize_field("id", &self.id)?;
        s.serialize_field("owner", &self.owner)?;
        s.serialize_field("publicKeyPem", &self.public_key_pem)?;
        s.serialize_field("type", "Key")?;
        s.end()
    }
}

#[derive(Debug, Clone)]
pub enum ApubMaybeId<T> {
    Id(String),
    Body(T),
}

pub trait ApubSigner {
    fn get_user_id(&self) -> String;
    fn get_private_key(&self) -> RsaPrivateKey;
    fn get_private_key_id(&self) -> String;
}
