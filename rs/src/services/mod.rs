use derive_builder::Builder;
use reqwest::Url;
use serde::{Deserialize, Serialize};
use uuid::{fmt::Simple, Uuid};

use derive_getters::Getters;

use crate::{
    models::{self, ApubSigner, ApubWebfingerResponse, PostPrivacy},
    utils::user::UserSpecifier,
};

use activitystreams::activity::{Accept, Follow};
use activitystreams::actor::Person;

use self::apub::render::ApubPerson;

pub mod apub;
pub mod db;
pub mod id;

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

pub trait UserCreateService {
    #[allow(async_fn_in_trait)]
    async fn create_user(
        &mut self,
        req: &UserCreateRequest,
    ) -> Result<UserCreateResult, ServiceError<UserCreateError>>;
    #[allow(async_fn_in_trait)]
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

pub trait LocalUserFinderService {
    #[allow(async_fn_in_trait)]
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

pub trait AllUserFinderService {
    #[allow(async_fn_in_trait)]
    async fn find_user_by_specifier(
        &mut self,
        spec: &UserSpecifier,
    ) -> Result<models::User, ServiceError<UserFindError>>;
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
}

#[derive(Debug, Clone, Builder)]
pub struct PostCreateRequestNormal {
    poster: UserSpecifier,
    content: String,
    privacy: PostPrivacy,
}

#[derive(Debug, Clone, Builder)]
pub struct PostCreateRequestRepost {
    poster: UserSpecifier,
    privacy: PostPrivacy,
    repost_of: Simple,
}

#[derive(Debug, Clone, Builder)]
pub struct PostCreateRequestQuote {
    poster: UserSpecifier,
    content: String,
    privacy: PostPrivacy,
    repost_of: Simple,
}

#[derive(Debug, Clone, Builder)]
pub struct PostCreateRequestReply {
    poster: UserSpecifier,
    content: String,
    privacy: PostPrivacy,
    reply_to: Simple,
}

#[derive(Debug)]
pub enum PostCreateError {
    PosterNotFound,
    RepostOfNotFound,
    ReplyToNotFound,
}

pub trait PostCreateService {
    #[allow(async_fn_in_trait)]
    async fn create_post(
        &mut self,
        req: &PostCreateRequest,
    ) -> Result<Simple, ServiceError<PostCreateError>>;
}

#[derive(Debug)]
pub enum AuthError {
    TokenNotSet,
}

pub trait UserAuthService {
    #[allow(async_fn_in_trait)]
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

#[derive(Debug, Clone, Getters)]
pub struct FollowRequestAccepted {
    pub follow_req_id: Uuid,
    pub follower_id: Uuid,
    pub followee_id: Uuid,
}

pub trait UserFollowService {
    #[allow(async_fn_in_trait)]
    async fn follow_user(
        &mut self,
        follower_spec: &UserSpecifier,
        followee_spec: &UserSpecifier,
    ) -> Result<(), ServiceError<FollowError>>;
    #[allow(async_fn_in_trait)]
    async fn unfollow_user(
        &mut self,
        follower_spec: &UserSpecifier,
        followee_spec: &UserSpecifier,
    ) -> Result<(), ServiceError<FollowError>>;
    #[allow(async_fn_in_trait)]
    async fn follow_request_accepted(
        &mut self,
        accepted_request: &FollowRequestSpecifier,
    ) -> Result<FollowRequestAccepted, ServiceError<FollowError>>;
}

#[derive(Debug)]
pub enum PostToInboxError {}

#[derive(Debug)]
pub enum ApubFetchUserError {
    NotFound,
}

#[derive(Debug)]
pub enum WebfingerError {
    ApiUrlNotFound,
    NotFound,
}

pub trait ApubRequestService {
    #[allow(async_fn_in_trait)]
    async fn post_to_inbox<T>(
        &mut self,
        url: impl Into<Url>,
        activity: T,
        actor: &impl ApubSigner,
    ) -> Result<(), ServiceError<PostToInboxError>>
    where
        T: Serialize;

    #[allow(async_fn_in_trait)]
    async fn fetch_user(
        &mut self,
        url: impl Into<Url>,
    ) -> Result<ApubPerson, ServiceError<ApubFetchUserError>>;
    #[allow(async_fn_in_trait)]
    async fn fetch_webfinger(
        &mut self,
        username: &str,
        host: &str,
    ) -> Result<ApubWebfingerResponse, ServiceError<WebfingerError>>;
}

#[derive(Debug, Clone, Serialize, Deserialize)]
pub enum BackgroundJob {
    RemoteFollowRequest(Uuid),
    RemoteFollowAccept(Uuid),
}

pub trait QueueService {
    #[allow(async_fn_in_trait)]
    async fn process_job(&self, job: BackgroundJob) -> Result<(), ServiceError<()>>;
}

#[derive(Debug, Clone)]
pub enum ApubFollowError {
    RequestNotFound,
}

pub trait ApubFollowService {
    #[allow(async_fn_in_trait)]
    async fn create_follow_request(&mut self, follow_req_id: Uuid)
        -> Result<Follow, anyhow::Error>;
    #[allow(async_fn_in_trait)]
    async fn create_follow_accept(&mut self, follow_req_id: Uuid) -> Result<Accept, anyhow::Error>;
}

#[derive(Debug, Clone)]
pub enum SignerError {
    UserNotFound,
    PrivateKeyNotSet,
}

pub trait SignerService {
    type Signer: ApubSigner;
    #[allow(async_fn_in_trait)]
    async fn fetch_signer(
        &mut self,
        user: &UserSpecifier,
    ) -> Result<Self::Signer, ServiceError<SignerError>>;
}
