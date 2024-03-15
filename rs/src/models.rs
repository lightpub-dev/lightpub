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
    fn bio(&self) -> String;
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

    fn bio(&self) -> String {
        self.bio.clone()
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

pub mod api_response {
    use derive_builder::Builder;
    use derive_getters::Getters;
    use uuid::fmt::Simple;

    use super::PostPrivacy;

    #[derive(Debug, Clone, Builder, Getters)]
    pub struct UserPostEntry {
        id: Simple,
        uri: String,
        author: PostAuthor,
        content: Option<String>,
        privacy: PostPrivacy,
        repost_of_id: Option<Simple>,
        reply_to_id: Option<Simple>,
        created_at: chrono::DateTime<chrono::Utc>,
        counts: PostCounts,
        reposted_by_you: bool,
        favorited_by_you: bool,
    }

    #[derive(Debug, Clone, Builder, Getters)]
    pub struct PostCounts {
        reactions: Vec<PostReaction>,
        replies: i64,
        reposts: i64,
        quotes: i64,
    }

    #[derive(Debug, Clone, Builder, Getters)]
    pub struct PostReaction {
        name: String,
        count: i64,
    }

    #[derive(Debug, Clone, Builder, Getters)]
    pub struct PostAuthor {
        id: Simple,
        uri: String,
        username: String,
        host: Option<String>,
        nickname: String,
    }
}

pub mod apub {
    use derive_builder::Builder;
    use derive_more::From;
    use serde::{Deserialize, Serialize};

    pub mod context {
        use serde::Serialize;

        #[derive(Serialize, Debug, Clone)]
        pub struct WithContext<T> {
            #[serde(rename = "@context")]
            context: Vec<String>,
            #[serde(flatten)]
            inner: T,
        }

        pub fn with_context<T>(inner: T) -> WithContext<T> {
            let context = vec![
                "https://www.w3.org/ns/activitystreams".to_string(),
                "https://w3id.org/security/v1".to_string(),
            ];
            WithContext { context, inner }
        }

        pub trait ContextAttachable {
            fn with_context(self) -> WithContext<Self>
            where
                Self: std::marker::Sized;
        }

        impl<T: std::fmt::Debug + Clone + Serialize> ContextAttachable for T {
            fn with_context(self) -> WithContext<Self> {
                with_context(self)
            }
        }
    }

    pub const PUBLIC: &str = "https://www.w3.org/ns/activitystreams#Public";

    pub trait HasId {
        fn get_id(&self) -> &str;
    }

    macro_rules! impl_id {
        ($t:ty) => {
            impl HasId for $t {
                fn get_id(&self) -> &str {
                    &self.id
                }
            }
        };
    }

    #[derive(Debug, Clone, Deserialize, Serialize, From)]
    #[serde(tag = "type")]
    pub enum Activity {
        Accept(AcceptActivity),
        Follow(FollowActivity),
        Create(CreateActivity),
        Announce(AnnounceActivity),
    }

    impl HasId for Activity {
        fn get_id(&self) -> &str {
            match self {
                Activity::Accept(a) => a.get_id(),
                Activity::Follow(a) => a.get_id(),
                Activity::Create(a) => a.get_id(),
                Activity::Announce(a) => a.get_id(),
            }
        }
    }

    #[derive(Debug, Clone, Deserialize, Serialize, Builder)]
    #[serde(rename_all = "camelCase")]
    pub struct AcceptActivity {
        pub id: String,
        pub actor: String,
        pub object: IdOrObject<FollowActivity>,
    }
    impl_id!(AcceptActivity);

    #[derive(Debug, Clone, Deserialize, Serialize, Builder)]
    #[serde(rename_all = "camelCase")]
    pub struct FollowActivity {
        pub id: String,
        pub actor: String,
        pub object: IdOrObject<Actor>,
    }
    impl_id!(FollowActivity);

    #[derive(Debug, Clone, Deserialize, Serialize)]
    #[serde(untagged)]
    pub enum IdOrObject<T> {
        Id(String),
        Object(T),
    }

    impl<T> HasId for IdOrObject<T>
    where
        T: HasId,
    {
        fn get_id(&self) -> &str {
            match self {
                IdOrObject::Id(id) => id,
                IdOrObject::Object(obj) => obj.get_id(),
            }
        }
    }

    #[derive(Debug, Clone, Deserialize, Serialize)]
    #[serde(tag = "type")]
    pub enum Actor {
        Person(Person),
    }

    impl HasId for Actor {
        fn get_id(&self) -> &str {
            match self {
                Actor::Person(p) => p.get_id(),
            }
        }
    }

    impl From<Person> for Actor {
        fn from(person: Person) -> Self {
            Actor::Person(person)
        }
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
        pub summary: Option<String>,
    }
    impl_id!(Person);

    #[derive(Debug, Clone, Deserialize, Serialize)]
    #[serde(tag = "type")]
    pub enum PublicKeyEnum {
        Key(PublicKey),
    }

    impl HasId for PublicKeyEnum {
        fn get_id(&self) -> &str {
            match self {
                PublicKeyEnum::Key(k) => k.get_id(),
            }
        }
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
    impl_id!(PublicKey);

    #[derive(Debug, Clone, Deserialize, Serialize)]
    #[serde(tag = "type")]
    pub enum CreatableObject {
        Note(Note),
    }

    impl HasId for CreatableObject {
        fn get_id(&self) -> &str {
            match self {
                CreatableObject::Note(n) => n.get_id(),
            }
        }
    }

    #[derive(Debug, Clone, Deserialize, Serialize, Builder)]
    #[serde(rename_all = "camelCase")]
    pub struct Note {
        pub id: String,
        pub attributed_to: String,
        pub content: String,
        pub to: Vec<String>,
        pub cc: Vec<String>,
        #[builder(default)]
        pub bto: Option<Vec<String>>,
        #[builder(default)]
        pub bcc: Option<Vec<String>>,
        pub published: chrono::DateTime<chrono::Utc>,
        #[builder(default)]
        pub in_reply_to: Option<Box<IdOrObject<CreatableObject>>>,
        #[builder(default)]
        pub tags: Option<Vec<TagEnum>>,
    }
    impl_id!(Note);

    #[derive(Debug, Clone, Deserialize, Serialize)]
    #[serde(tag = "type")]
    pub enum TagEnum {
        Mention(Mention),
        Hashtag(Hashtag),
    }

    #[derive(Debug, Clone, Deserialize, Serialize)]
    #[serde(rename_all = "camelCase")]
    pub struct Mention {
        pub href: String,
        pub name: String,
    }

    #[derive(Debug, Clone, Deserialize, Serialize)]
    #[serde(rename_all = "camelCase")]
    pub struct Hashtag {
        pub name: String,
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
    impl_id!(CreateActivity);

    #[derive(Debug, Clone, Deserialize, Serialize, Builder)]
    #[serde(rename_all = "camelCase")]
    pub struct AnnounceActivity {
        pub id: String,
        pub actor: String,
        pub object: IdOrObject<CreatableObject>,
        pub published: chrono::DateTime<chrono::Utc>,
        pub to: Vec<String>,
        pub cc: Vec<String>,
        pub bto: Option<Vec<String>>,
        pub bcc: Option<Vec<String>>,
    }
    impl_id!(AnnounceActivity);
}

pub mod http {
    #[derive(Debug, Clone, Copy, PartialEq, Eq)]
    pub enum Method {
        GET,
        POST,
        PUT,
        PATCH,
        DELETE,
    }

    impl Method {
        pub fn as_str(&self) -> &'static str {
            use Method::*;
            match self {
                GET => "GET",
                POST => "POST",
                PUT => "PUT",
                PATCH => "PATCH",
                DELETE => "DELETE",
            }
        }

        pub fn from_reqwest(m: &reqwest::Method) -> Self {
            use reqwest::Method as M;
            use Method::*;
            match *m {
                M::GET => GET,
                M::POST => POST,
                M::PUT => PUT,
                M::PATCH => PATCH,
                M::DELETE => DELETE,
                _ => unimplemented!(),
            }
        }

        pub fn from_actix(m: &actix_web::http::Method) -> Self {
            use actix_web::http::Method as M;
            use Method::*;
            match *m {
                M::GET => GET,
                M::POST => POST,
                M::PUT => PUT,
                M::PATCH => PATCH,
                M::DELETE => DELETE,
                _ => unimplemented!(),
            }
        }
    }

    #[derive(Debug)]
    pub enum HeaderMapWrapper<'a> {
        Reqwest(&'a reqwest::header::HeaderMap),
        Actix(&'a actix_web::http::header::HeaderMap),
    }

    impl<'a> HeaderMapWrapper<'a> {
        pub fn from_reqwest(h: &'a reqwest::header::HeaderMap) -> Self {
            HeaderMapWrapper::Reqwest(h)
        }

        pub fn from_actix(h: &'a actix_web::http::header::HeaderMap) -> Self {
            HeaderMapWrapper::Actix(h)
        }

        pub fn get(&self, key: &str) -> Option<&str> {
            use HeaderMapWrapper::*;
            match self {
                Reqwest(h) => h.get(key).map(|v| v.to_str().unwrap()),
                Actix(h) => h.get(key).map(|v| v.to_str().unwrap()),
            }
        }
    }
}
