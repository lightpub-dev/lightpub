use async_trait::async_trait;

use crate::holder;

use super::{auth::AuthTokenRepository, follow::FollowRepository, user::UserRepository};

pub trait RepositoryManager {
    fn user_repository(&self) -> holder!(UserRepository);
    fn auth_token_repository(&self) -> holder!(AuthTokenRepository);
    fn follow_repository(&self) -> holder!(FollowRepository);
}

#[async_trait]
pub trait UnitOfWork {
    fn repository_manager(&self) -> holder!(RepositoryManager);
    async fn commit(&mut self) -> Result<(), anyhow::Error>;
    async fn rollback(&mut self) -> Result<(), anyhow::Error>;
}
