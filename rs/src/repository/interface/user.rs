use async_trait::async_trait;

use crate::domain::model::user::{User, UserId, Username};

use super::RepositoryError;

#[async_trait]
pub trait UserRepository {
    async fn create(&mut self, user: &User) -> Result<UserId, RepositoryError>;
    async fn find_by_id(&mut self, user_id: &UserId) -> Result<Option<User>, RepositoryError>;
    async fn find_by_username_and_host(
        &mut self,
        username: &Username,
        host: Option<&str>,
    ) -> Result<Option<User>, RepositoryError>;
}
