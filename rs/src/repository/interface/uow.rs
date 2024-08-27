use async_trait::async_trait;

use crate::holder;

use super::{
    auth::AuthTokenRepository, follow::FollowRepository, post::PostRepository, user::UserRepository,
};

#[async_trait]
pub trait RepositoryManager {
    fn user_repository<'a>(&'a mut self) -> Box<dyn UserRepository + 'a>;
    fn auth_token_repository<'a>(&'a mut self) -> Box<dyn AuthTokenRepository + 'a>;
    fn follow_repository<'a>(&'a mut self) -> Box<dyn FollowRepository + 'a>;
    fn post_repository<'a>(&'a mut self) -> Box<dyn PostRepository + 'a>;

    async fn commit(self) -> Result<(), anyhow::Error>;
    async fn rollback(self) -> Result<(), anyhow::Error>;
}

#[async_trait]
pub trait UnitOfWork {
    async fn repository_manager(
        &mut self,
    ) -> Result<Box<dyn RepositoryManager + '_>, anyhow::Error>;
}
