use derive_builder::Builder;
use uuid::fmt::Simple;

use derive_getters::Getters;

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

impl<T, S: MiscError + 'static> From<S> for ServiceError<T> {
    fn from(value: S) -> Self {
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
