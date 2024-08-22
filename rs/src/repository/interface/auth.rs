use async_trait::async_trait;

use crate::domain::model::auth::AuthToken;

use super::RepositoryError;

#[async_trait]
pub trait AuthTokenRepository {
    async fn create(&mut self, token: &AuthToken) -> Result<(), RepositoryError>;
    async fn exists(&mut self, token: &AuthToken) -> Result<bool, RepositoryError>;
}
