use thiserror::Error;

pub mod auth;
pub mod follow;
pub mod post;
pub mod uow;
pub mod user;

#[derive(Debug, Error)]
pub enum RepositoryError {
    #[error("User creation error: {0}")]
    UserCreationError(String),
    #[error("User find error: {0}")]
    UserFindError(String),
}
