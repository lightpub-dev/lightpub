use derive_builder::Builder;
use derive_getters::Getters;
use rsa::RsaPrivateKey;
use serde::Deserialize;
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

    fn created_at_fixed_offset(&self) -> chrono::DateTime<chrono::FixedOffset> {
        self.created_at()
            .with_timezone(&chrono::FixedOffset::east_opt(0).unwrap())
    }
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
            Followers => "follower",
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

pub trait ApubSigner {
    fn get_user_id(&self) -> String;
    fn get_private_key(&self) -> RsaPrivateKey;
    fn get_private_key_id(&self) -> String;
}

pub mod apub {
    use derive_builder::Builder;
    use serde::{Deserialize, Serialize};

    #[derive(Debug, Clone, Deserialize, Serialize)]
    #[serde(tag = "type")]
    pub enum Activity {
        Accept(AcceptActivity),
        Follow(FollowActivity),
        Create(CreateActivity),
        Announce(AnnounceActivity),
    }

    #[derive(Debug, Clone, Deserialize, Serialize, Builder)]
    #[serde(rename_all = "camelCase")]
    pub struct AcceptActivity {
        id: String,
        actor: String,
        object: IdOrObject<FollowActivity>,
    }

    #[derive(Debug, Clone, Deserialize, Serialize, Builder)]
    #[serde(rename_all = "camelCase")]
    pub struct FollowActivity {
        id: String,
        actor: String,
        object: IdOrObject<Person>,
    }

    #[derive(Debug, Clone, Deserialize, Serialize)]
    #[serde(untagged)]
    pub enum IdOrObject<T> {
        Id(String),
        Object(T),
    }

    #[derive(Debug, Clone, Deserialize, Serialize, Builder)]
    #[serde(rename_all = "camelCase")]
    pub struct Person {
        pub id: String,
        pub name: String,
        pub inbox: String,
        pub outbox: String,
        pub shared_inbox: Option<String>,
        pub followers: Option<String>,
        pub following: Option<String>,
        pub liked: Option<String>,
        pub preferred_username: String,
        pub public_key: PublicKeyEnum,
    }

    #[derive(Debug, Clone, Deserialize, Serialize)]
    #[serde(tag = "type")]
    pub enum PublicKeyEnum {
        Key(PublicKey),
    }

    impl From<PublicKey> for PublicKeyEnum {
        fn from(key: PublicKey) -> Self {
            PublicKeyEnum::Key(key)
        }
    }

    #[derive(Debug, Clone, Deserialize, Serialize, Builder)]
    #[serde(rename_all = "camelCase")]
    pub struct PublicKey {
        pub id: String,
        pub owner: String,
        pub public_key_pem: String,
    }

    #[derive(Debug, Clone, Deserialize, Serialize)]
    #[serde(tag = "type")]
    pub enum CreatableObject {
        Note(Note),
    }

    #[derive(Debug, Clone, Deserialize, Serialize, Builder)]
    #[serde(rename_all = "camelCase")]
    pub struct Note {
        pub id: String,
        pub attributed_to: String,
        pub content: String,
        pub to: Vec<String>,
        pub cc: Vec<String>,
        pub bto: Option<Vec<String>>,
        pub bcc: Option<Vec<String>>,
        pub published_at: chrono::DateTime<chrono::Utc>,
    }

    #[derive(Debug, Clone, Deserialize, Serialize, Builder)]
    #[serde(rename_all = "camelCase")]
    pub struct CreateActivity {
        pub id: String,
        pub actor: String,
        pub object: IdOrObject<CreatableObject>,
        pub to: Vec<String>,
        pub cc: Vec<String>,
        pub bto: Option<Vec<String>>,
        pub bcc: Option<Vec<String>>,
    }

    #[derive(Debug, Clone, Deserialize, Serialize, Builder)]
    #[serde(rename_all = "camelCase")]
    pub struct AnnounceActivity {
        pub id: String,
        pub actor: String,
        pub object: IdOrObject<CreatableObject>,
    }
}
