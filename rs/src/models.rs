use serde::Deserialize;

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

#[derive(Debug)]
pub struct ApubWebfingerResponse {
    api_url: String,
    profile_url: Option<String>,
}

#[derive(Debug)]
pub struct ApubPerson {}

#[derive(Debug)]
pub enum ApubActivity {}

#[derive(Debug)]
pub struct ApubActor {}
