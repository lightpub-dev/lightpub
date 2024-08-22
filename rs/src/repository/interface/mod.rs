use thiserror::Error;

pub mod auth;
pub mod user;

#[derive(Debug, Error)]
pub enum RepositoryError {
    #[error("User creation error: {0}")]
    UserCreationError(String),
    #[error("User find error: {0}")]
    UserFindError(String),
}
