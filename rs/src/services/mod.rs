use async_trait::async_trait;
use derive_builder::Builder;
use derive_more::From;
use serde::{Deserialize, Serialize};
use uuid::{fmt::Simple, Uuid};

use derive_getters::Getters;

use crate::{
    models::{self, ApubSigner, ApubWebfingerResponse, PostPrivacy},
    utils::{post::PostSpecifier, user::UserSpecifier},
};

use activitystreams::activity::{Accept, Follow};

use self::apub::render::{ApubNote, ApubNoteCreate, ApubPerson};

pub mod apub;
pub mod db;
pub mod id;

pub type Holder<T> = Box<T>;
#[macro_export]
macro_rules! holder {
    ($t:tt) => {
        crate::services::Holder<dyn $t + Send + Sync>
    };
}

pub trait MiscError: std::fmt::Debug + Send + Sync {
    fn message(&self) -> &str;
    fn status_code(&self) -> i32;
}

impl MiscError for sqlx::Error {
    fn message(&self) -> &str {
        "internal server error"
    }

    fn status_code(&self) -> i32 {
        500
    }
}

impl MiscError for reqwest::Error {
    fn message(&self) -> &str {
        "internal server error"
    }

    fn status_code(&self) -> i32 {
        500
    }
}

#[derive(Debug, Clone, Builder)]
pub struct UserCreateRequest {
    username: String,
    nickname: String,
    password: String,
}

#[derive(Debug, Clone, Getters)]
pub struct UserCreateResult {
    user_id: Simple,
}

#[derive(Debug)]
pub enum ServiceError<T> {
    SpecificError(T),
    MiscError(Box<dyn MiscError>),
}

impl<T: std::fmt::Debug> ServiceError<T> {
    pub fn from_se(e: T) -> Self {
        ServiceError::SpecificError(e)
    }

    pub fn convert<S>(self) -> ServiceError<S> {
        match self {
            ServiceError::SpecificError(e) => panic!("unhandled error: {:?}", e),
            ServiceError::MiscError(e) => ServiceError::MiscError(e),
        }
    }
}

impl<T> From<Box<dyn MiscError>> for ServiceError<T> {
    fn from(value: Box<dyn MiscError>) -> Self {
        ServiceError::MiscError(value)
    }
}

impl<T> From<sqlx::Error> for ServiceError<T> {
    fn from(value: sqlx::Error) -> Self {
        ServiceError::MiscError(Box::new(value))
    }
}

#[derive(Debug)]
pub enum UserCreateError {
    UsernameConflict,
}

#[derive(Debug, Clone, Builder)]
pub struct UserLoginRequest {
    username: String,
    password: String,
}

#[derive(Debug, Clone, Builder, Getters)]
pub struct UserLoginResult {
    user_token: Simple,
}

#[derive(Debug)]
pub enum UserLoginError {
    AuthFailed,
}

#[async_trait]
pub trait UserCreateService {
    async fn create_user(
        &mut self,
        req: &UserCreateRequest,
    ) -> Result<UserCreateResult, ServiceError<UserCreateError>>;

    async fn login_user(
        &mut self,
        req: &UserLoginRequest,
    ) -> Result<UserLoginResult, ServiceError<UserLoginError>>;
}

#[derive(Debug, Clone)]
pub enum LocalUserFindError {
    UserNotFound,
    NotLocalUser,
}

#[async_trait]
pub trait LocalUserFinderService {
    async fn find_user_by_specifier(
        &mut self,
        spec: &UserSpecifier,
    ) -> Result<models::User, ServiceError<LocalUserFindError>>;
}

#[derive(Debug, Clone)]
pub enum UserFindError {
    UserNotFound,
    RemoteError,
}

#[async_trait]
pub trait AllUserFinderService {
    async fn find_user_by_specifier(
        &mut self,
        spec: &UserSpecifier,
    ) -> Result<models::User, ServiceError<UserFindError>>;

    async fn find_followers_inboxes(
        &mut self,
        user: &UserSpecifier,
    ) -> Result<Vec<InboxPair>, ServiceError<UserFindError>>;
}

#[derive(Debug, Clone, Getters)]
pub struct InboxPair {
    inbox: Option<String>,
    shared_inbox: Option<String>,
}

#[derive(Debug, Clone)]
pub enum PostCreateRequest {
    Normal(PostCreateRequestNormal),
    Repost(PostCreateRequestRepost),
    Quote(PostCreateRequestQuote),
    Reply(PostCreateRequestReply),
}

impl PostCreateRequest {
    pub fn poster(&self) -> &UserSpecifier {
        match self {
            PostCreateRequest::Normal(r) => &r.poster,
            PostCreateRequest::Repost(r) => &r.poster,
            PostCreateRequest::Quote(r) => &r.poster,
            PostCreateRequest::Reply(r) => &r.poster,
        }
    }

    pub fn privacy(&self) -> PostPrivacy {
        match self {
            PostCreateRequest::Normal(r) => r.privacy,
            PostCreateRequest::Repost(r) => r.privacy,
            PostCreateRequest::Quote(r) => r.privacy,
            PostCreateRequest::Reply(r) => r.privacy,
        }
    }

    pub fn uri(&self) -> &Option<String> {
        match self {
            PostCreateRequest::Normal(r) => &r.uri,
            PostCreateRequest::Repost(r) => &r.uri,
            PostCreateRequest::Quote(r) => &r.uri,
            PostCreateRequest::Reply(r) => &r.uri,
        }
    }
}

#[derive(Debug, Clone)]
pub enum ApubNoteError {
    IDNotFound,
    AttributedToNotFound,
    ContentNotFound,
}

impl TryFrom<ApubNote> for PostCreateRequest {
    type Error = ApubNoteError;

    fn try_from(value: ApubNote) -> Result<Self, Self::Error> {
        let id = value
            .as_ref()
            .get_id()
            .map(|s| s.to_string())
            .ok_or_else(|| ApubNoteError::IDNotFound)?;

        let attributed_to = value
            .as_ref()
            .get_attributed_to_xsd_any_uri()
            .map(|s| UserSpecifier::from_url(s.to_string()))
            .ok_or_else(|| ApubNoteError::AttributedToNotFound)?;

        let content = value
            .as_ref()
            .get_content_xsd_string()
            .map(|s| s.to_string())
            .ok_or_else(|| ApubNoteError::ContentNotFound)?;

        let reply_to_id = value
            .as_ref()
            .get_in_reply_to_xsd_any_uri()
            .map(|s| PostSpecifier::from_uri(s.to_string()));

        let privacy = {
            let to = value
                .as_ref()
                .get_many_to_xsd_any_uris()
                .map(|v| v.collect())
                .unwrap_or_else(|| vec![]);
            let cc = value
                .as_ref()
                .get_many_cc_xsd_any_uris()
                .map(|v| v.collect())
                .unwrap_or_else(|| vec![]);
            let bto = value
                .as_ref()
                .get_many_bto_xsd_any_uris()
                .map(|v| v.collect())
                .unwrap_or_else(|| vec![]);
            let bcc = value
                .as_ref()
                .get_many_bcc_xsd_any_uris()
                .map(|v| v.collect())
                .unwrap_or_else(|| vec![]);

            // combine to and bto, cc and bcc
            let to = to.into_iter().chain(bto.into_iter()).collect::<Vec<_>>();
            let cc = cc.into_iter().chain(bcc.into_iter()).collect::<Vec<_>>();
            // remove duplicates
            let to: Vec<_> = to
                .into_iter()
                .collect::<std::collections::HashSet<_>>()
                .into_iter()
                .collect();
            let cc: Vec<_> = cc
                .into_iter()
                .collect::<std::collections::HashSet<_>>()
                .into_iter()
                .collect();

            // to contains #public => public
            // cc contains #public => unlisted
            // to contains /followers => followers
            // otherwise => private
            if to.contains(&&activitystreams::public()) {
                PostPrivacy::Public
            } else if cc.contains(&&activitystreams::public()) {
                PostPrivacy::Unlisted
            } else {
                let to_contains_followers = to.iter().any(|s| s.as_str().ends_with("/followers"));
                if to_contains_followers {
                    PostPrivacy::Followers
                } else {
                    PostPrivacy::Private
                }
            }
        };

        if let Some(reply_to_id) = reply_to_id {
            Ok(PostCreateRequest::Reply(
                PostCreateRequestReplyBuilder::default()
                    .uri(id)
                    .poster(attributed_to)
                    .content(content)
                    .privacy(privacy)
                    .reply_to(reply_to_id)
                    .build()
                    .unwrap(),
            ))
        } else {
            Ok(PostCreateRequest::Normal(
                PostCreateRequestNormalBuilder::default()
                    .uri(id)
                    .poster(attributed_to)
                    .content(content)
                    .privacy(privacy)
                    .build()
                    .unwrap(),
            ))
        }
    }
}

#[derive(Debug, Clone, Builder)]
pub struct PostCreateRequestNormal {
    poster: UserSpecifier,
    #[builder(default, setter(into, strip_option))]
    uri: Option<String>,
    content: String,
    privacy: PostPrivacy,
}

#[derive(Debug, Clone, Builder)]
pub struct PostCreateRequestRepost {
    poster: UserSpecifier,
    #[builder(default, setter(into, strip_option))]
    uri: Option<String>,
    privacy: PostPrivacy,
    repost_of: PostSpecifier,
}

#[derive(Debug, Clone, Builder)]
pub struct PostCreateRequestQuote {
    poster: UserSpecifier,
    #[builder(default, setter(into, strip_option))]
    uri: Option<String>,
    content: String,
    privacy: PostPrivacy,
    repost_of: PostSpecifier,
}

#[derive(Debug, Clone, Builder)]
pub struct PostCreateRequestReply {
    poster: UserSpecifier,
    #[builder(default, setter(into, strip_option))]
    uri: Option<String>,
    content: String,
    privacy: PostPrivacy,
    reply_to: PostSpecifier,
}

#[derive(Debug, Clone)]
pub enum PostCreateError {
    PosterNotFound,
    RepostOfNotFound,
    ReplyToNotFound,
    AlreadyExists,
}

#[async_trait]
pub trait PostCreateService {
    async fn create_post(
        &mut self,
        req: &PostCreateRequest,
    ) -> Result<Simple, ServiceError<PostCreateError>>;
}

#[derive(Debug)]
pub enum AuthError {
    TokenNotSet,
}

#[async_trait]
pub trait UserAuthService {
    async fn authenticate_user(
        &mut self,
        token: &str,
    ) -> Result<models::User, ServiceError<AuthError>>;
}

#[derive(Debug, Clone)]
pub enum FollowError {
    FollowerNotFound,
    FolloweeNotFound,
    RequestNotFound,
}

#[derive(Debug, Clone)]
pub enum FollowRequestSpecifier {
    LocalURI(String),
    ActorPair(UserSpecifier, UserSpecifier),
}

#[derive(Debug, Clone)]
pub enum IncomingFollowRequest {
    ActorPair(String, UserSpecifier, UserSpecifier),
}

#[derive(Debug, Clone, Getters)]
pub struct FollowRequestAccepted {
    pub follow_req_id: Uuid,
    pub follower_id: Uuid,
    pub followee_id: Uuid,
}

#[async_trait]
pub trait UserFollowService {
    async fn follow_user(
        &mut self,
        follower_spec: &UserSpecifier,
        followee_spec: &UserSpecifier,
    ) -> Result<(), ServiceError<FollowError>>;

    async fn unfollow_user(
        &mut self,
        follower_spec: &UserSpecifier,
        followee_spec: &UserSpecifier,
    ) -> Result<(), ServiceError<FollowError>>;

    async fn follow_request_accepted(
        &mut self,
        accepted_request: &FollowRequestSpecifier,
    ) -> Result<FollowRequestAccepted, ServiceError<FollowError>>;

    async fn incoming_follow_request(
        &mut self,
        incoming_follow_request: &IncomingFollowRequest,
    ) -> Result<(), ServiceError<FollowError>>;
}

#[derive(Debug)]
pub enum PostToInboxError {}

#[derive(Debug)]
pub enum ApubFetchUserError {
    NotFound,
}

#[derive(Debug)]
pub enum ApubFetchPostError {
    NotFound,
}

#[derive(Debug)]
pub enum WebfingerError {
    ApiUrlNotFound,
    NotFound,
}

#[derive(Debug, Clone, From)]
pub enum SendActivity {
    Follow(activitystreams::activity::Follow),
    Accept(activitystreams::activity::Accept),
    Create(ApubNoteCreate),
}

impl SendActivity {
    pub fn to_json(&self) -> String {
        use SendActivity::*;
        match self {
            Follow(f) => serde_json::to_string(f).unwrap(),
            Accept(a) => serde_json::to_string(a).unwrap(),
            Create(c) => serde_json::to_string(c).unwrap(),
        }
    }
}

#[async_trait]
pub trait ApubRequestService {
    async fn post_to_inbox(
        &mut self,
        url: &str,
        activity: &SendActivity,
        actor: holder!(ApubSigner),
    ) -> Result<(), ServiceError<PostToInboxError>>;

    async fn fetch_user(
        &mut self,
        url: &str,
    ) -> Result<ApubPerson, ServiceError<ApubFetchUserError>>;

    async fn fetch_webfinger(
        &mut self,
        username: &str,
        host: &str,
    ) -> Result<ApubWebfingerResponse, ServiceError<WebfingerError>>;

    async fn fetch_post(&mut self, url: &str)
        -> Result<ApubNote, ServiceError<ApubFetchPostError>>;
}

#[derive(Debug, Clone, Serialize, Deserialize)]
pub enum BackgroundJob {
    RemoteFollowRequest(Uuid),
    RemoteFollowAccept(Uuid),
}

#[async_trait]
pub trait QueueService {
    async fn process_job(&self, job: BackgroundJob) -> Result<(), ServiceError<()>>;
}

#[derive(Debug, Clone)]
pub enum ApubFollowError {
    RequestNotFound,
}

#[async_trait]
pub trait ApubFollowService {
    async fn create_follow_request(&mut self, follow_req_id: Uuid)
        -> Result<Follow, anyhow::Error>;

    async fn create_follow_accept(&mut self, follow_req_id: Uuid) -> Result<Accept, anyhow::Error>;
}

#[derive(Debug, Clone)]
pub enum SignerError {
    UserNotFound,
    PrivateKeyNotSet,
}

#[async_trait]
pub trait SignerService {
    async fn fetch_signer(
        &mut self,
        user: &UserSpecifier,
    ) -> Result<holder!(ApubSigner), ServiceError<SignerError>>;
}
