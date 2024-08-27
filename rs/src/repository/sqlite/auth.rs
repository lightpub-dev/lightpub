use async_trait::async_trait;

use crate::{
    domain::model::{auth::AuthToken, user::UserId},
    repository::interface::{auth::AuthTokenRepository, RepositoryError},
};

use super::SqliteRepository;

#[async_trait]
impl AuthTokenRepository for SqliteRepository<'_> {
    async fn create(&mut self, token: &AuthToken, user_id: UserId) -> Result<(), RepositoryError> {
        todo!()
    }
    async fn find_by_token(&mut self, token: &str) -> Result<Option<AuthToken>, RepositoryError> {
        todo!()
    }
}
