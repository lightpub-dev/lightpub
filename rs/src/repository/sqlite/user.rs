use async_trait::async_trait;

use crate::{
    domain::model::user::{User, UserId, Username},
    repository::interface::{user::UserRepository, RepositoryError},
};

use super::{IsUuid, SqliteRepository};

impl IsUuid for UserId {
    fn to_uuid(&self) -> uuid::Uuid {
        self.id()
    }

    fn from_uuid(uuid: uuid::Uuid) -> Self {
        Self::from_uuid(uuid)
    }
}

#[async_trait]
impl UserRepository for SqliteRepository<'_> {
    async fn create(&mut self, user: &User) -> Result<UserId, RepositoryError> {
        todo!()
    }
    async fn find_by_id(&mut self, user_id: &UserId) -> Result<Option<User>, RepositoryError> {
        todo!()
    }
    async fn find_by_username_and_host(
        &mut self,
        username: &Username,
        host: Option<&str>,
    ) -> Result<Option<User>, RepositoryError> {
        todo!()
    }
}
