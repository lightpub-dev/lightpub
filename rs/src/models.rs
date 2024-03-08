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

impl HasRemoteUri for User {
    fn get_local_id(&self) -> String {
        self.id.to_string()
    }

    fn get_remote_uri(&self) -> Option<String> {
        self.uri.clone()
    }
}

impl HasRemoteUri for &User {
    fn get_local_id(&self) -> String {
        self.id.to_string()
    }

    fn get_remote_uri(&self) -> Option<String> {
        self.uri.clone()
    }
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

impl ApubRenderableUser for User {
    fn id(&self) -> Simple {
        self.id
    }

    fn uri(&self) -> Option<String> {
        self.uri.clone()
    }

    fn username(&self) -> String {
        self.username.clone()
    }

    fn nickname(&self) -> String {
        self.nickname.clone()
    }

    fn public_key(&self) -> Option<String> {
        self.public_key.clone()
    }

    fn created_at(&self) -> chrono::DateTime<chrono::Utc> {
        chrono::DateTime::<chrono::Utc>::from_naive_utc_and_offset(self.created_at, chrono::Utc)
    }
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

#[derive(Debug, Clone, Serialize)]
pub struct ApubPayload<T> {
    #[serde(rename = "@context")]
    pub context: Vec<String>,
    #[serde(flatten)]
    pub content: T,
}

pub struct ApubPayloadBuilder<T> {
    context: Vec<String>,
    content: T,
}

impl<T> ApubPayloadBuilder<T> {
    pub fn new(content: T) -> Self {
        Self {
            context: vec![],
            content,
        }
    }

    pub fn with_context(mut self, context: impl Into<String>) -> Self {
        self.context.push(context.into());
        self
    }

    pub fn build(self) -> ApubPayload<T> {
        ApubPayload {
            context: self.context,
            content: self.content,
        }
    }
}

impl<T: Serialize> ApubPayload<T> {
    pub fn to_json(&self) -> String {
        serde_json::to_string(self).unwrap()
    }
}

#[derive(Debug, Clone, Serialize)]
#[serde(untagged)]
pub enum ApubActivity {
    Follow(ApubFollow),
    // Accept(ApubAccept),
    // Reject(ApubReject),
    // Undo(ApubUndo),
}

impl ApubActivity {
    pub fn to_json(&self) -> String {
        use ApubActivity::*;
        match self {
            Follow(f) => serde_json::to_string(f).unwrap(),
            _ => todo!("implement the rest of the activity types"),
        }
    }
}

#[derive(Debug, Builder, Clone)]
pub struct ApubFollow {
    pub id: String,
    pub actor: ApubMaybeId<ApubPerson>,
    pub object: ApubMaybeId<ApubPerson>,
}

impl Serialize for ApubFollow {
    fn serialize<S>(&self, serializer: S) -> Result<S::Ok, S::Error>
    where
        S: serde::Serializer,
    {
        let mut state = serializer.serialize_struct("ApubFollow", 4)?;
        state.serialize_field("id", &self.id)?;
        state.serialize_field("actor", &self.actor)?;
        state.serialize_field("object", &self.object)?;
        state.serialize_field("type", "Follow")?;
        state.end()
    }
}

#[derive(Debug, Builder, Clone)]
pub struct ApubAccept {
    pub id: String,
    pub actor: ApubMaybeId<ApubPerson>,
    pub object: ApubMaybeId<ApubFollow>,
}

#[derive(Debug, Builder, Clone)]
pub struct ApubReject {
    pub id: String,
    pub actor: ApubMaybeId<ApubPerson>,
    pub object: ApubMaybeId<ApubFollow>,
}

#[derive(Debug, Builder, Clone)]
pub struct ApubUndo {
    pub id: String,
    pub actor: ApubMaybeId<ApubPerson>,
    pub object: ApubMaybeId<ApubFollow>,
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

#[derive(Debug, Deserialize, Builder, Clone, Getters)]
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

impl Serialize for ApubPerson {
    fn serialize<S>(&self, serializer: S) -> Result<S::Ok, S::Error>
    where
        S: serde::Serializer,
    {
        let mut s = serializer.serialize_struct("ApubPerson", 9)?;
        s.serialize_field("id", &self.id)?;
        s.serialize_field("inbox", &self.inbox)?;
        s.serialize_field("outbox", &self.outbox)?;
        s.serialize_field("following", &self.following)?;
        s.serialize_field("followers", &self.followers)?;
        s.serialize_field("liked", &self.liked)?;
        s.serialize_field("type", "Person")?;
        s.serialize_field("preferredUsername", &self.preferred_username)?;
        s.serialize_field("name", &self.name)?;
        s.serialize_field("publicKey", &self.public_key)?;
        s.serialize_field("sharedInbox", &self.shared_inbox)?;
        s.end()
    }
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

impl<T> From<String> for ApubMaybeId<T> {
    fn from(s: String) -> Self {
        ApubMaybeId::Id(s)
    }
}

impl<T> Serialize for ApubMaybeId<T>
where
    T: Serialize,
{
    fn serialize<S>(&self, serializer: S) -> Result<S::Ok, S::Error>
    where
        S: serde::Serializer,
    {
        match self {
            ApubMaybeId::Id(id) => id.serialize(serializer),
            ApubMaybeId::Body(body) => body.serialize(serializer),
        }
    }
}

pub trait ApubSigner {
    fn get_user_id(&self) -> String;
    fn get_private_key(&self) -> RsaPrivateKey;
    fn get_private_key_id(&self) -> String;
}
