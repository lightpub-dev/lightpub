use async_trait::async_trait;

use crate::domain::model::{auth::AuthToken, user::UserId};

use super::RepositoryError;

#[async_trait]
pub trait AuthTokenRepository {
    async fn create(&mut self, token: &AuthToken, user_id: UserId) -> Result<(), RepositoryError>;
    async fn find_by_token(&mut self, token: &str) -> Result<Option<AuthToken>, RepositoryError>;
}
