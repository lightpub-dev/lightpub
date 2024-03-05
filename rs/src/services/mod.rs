use derive_builder::Builder;
use uuid::fmt::Simple;

use derive_getters::Getters;

use crate::{
    models::{self, PostPrivacy},
    utils::user::UserSpecifier,
};

pub mod db;

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

impl<T> ServiceError<T> {
    pub fn from_se(e: T) -> Self {
        ServiceError::SpecificError(e)
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

pub trait LocalUserFinderService {
    async fn find_user_by_specifier(
        &mut self,
        spec: &UserSpecifier,
    ) -> Result<models::User, ServiceError<LocalUserFindError>>;
}

#[derive(Debug, Clone, Builder)]
pub struct PostCreateRequest {
    poster: UserSpecifier,
    content: String,
    privacy: PostPrivacy,
}

#[derive(Debug)]
pub enum PostCreateError {
    PosterNotFound,
    RepostOfNotFound,
    ReplyToNotFound,
}

pub trait PostCreateService {
    async fn create_post(
        &mut self,
        req: &PostCreateRequest,
    ) -> Result<(), ServiceError<PostCreateError>>;
}

#[derive(Debug)]
pub enum AuthError {
    TokenNotSet,
}

pub trait UserAuthService {
    async fn authenticate_user(
        &mut self,
        token: &str,
    ) -> Result<models::User, ServiceError<AuthError>>;
}
