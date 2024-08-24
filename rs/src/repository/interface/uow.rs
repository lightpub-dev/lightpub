use async_trait::async_trait;

use crate::holder;

use super::{
    auth::AuthTokenRepository, follow::FollowRepository, post::PostRepository, user::UserRepository,
};

pub trait RepositoryManager {
    fn user_repository(&self) -> holder!(UserRepository);
    fn auth_token_repository(&self) -> holder!(AuthTokenRepository);
    fn follow_repository(&self) -> holder!(FollowRepository);
    fn post_repository(&self) -> holder!(PostRepository);
}

#[async_trait]
pub trait UnitOfWork {
    fn repository_manager(&self) -> holder!(RepositoryManager);
    async fn commit(&mut self) -> Result<(), anyhow::Error>;
    async fn rollback(&mut self) -> Result<(), anyhow::Error>;
}
