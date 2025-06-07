use std::sync::Arc;

use derive_more::Constructor;

use crate::domain::{
    repositories::{
        follow::UserFollowRepository,
        user::{UserAuthRepository, UserRepository},
    },
    services::auth::PasswordHashService,
};

#[derive(Debug, Clone, Constructor)]
pub struct ServiceLocator {
    user_repository: Arc<dyn UserRepository + Send + Sync>,
    user_auth_repository: Arc<dyn UserAuthRepository + Send + Sync>,
    follow_repository: Arc<dyn UserFollowRepository + Send + Sync>,

    hash_service: Arc<dyn PasswordHashService + Send + Sync>,
}

impl ServiceLocator {
    pub fn user_repository(&self) -> Arc<dyn UserRepository> {
        self.user_repository.as_ref().clone()
    }

    pub fn user_auth_repository(&self) -> Arc<dyn UserAuthRepository> {
        self.user_auth_repository.as_ref().clone()
    }

    pub fn follow_repository(&self) -> Arc<dyn UserFollowRepository> {
        self.follow_repository.as_ref().clone()
    }

    pub fn hash_service(&self) -> Arc<dyn PasswordHashService> {
        self.hash_service.as_ref().clone()
    }
}
