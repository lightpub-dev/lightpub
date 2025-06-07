use async_trait::async_trait;

use crate::{
    ServiceResult,
    domain::models::{
        apub::ActorID,
        user::{ApubActorEntity, UserAuthEntity, UserEntity, UserID},
    },
};

#[async_trait]
pub trait UserAuthRepository {
    async fn get_auth_by_user_id(&self, user_id: UserID) -> ServiceResult<Option<UserAuthEntity>>;

    async fn save(&self, user_auth: &mut UserAuthEntity) -> ServiceResult<()>;
}

#[async_trait]
pub trait UserRepository {
    async fn get_user_by_id(&self, user_id: UserID) -> ServiceResult<Option<UserEntity>>;
    async fn get_total_users_count(&self) -> ServiceResult<u64>;

    async fn save(&self, user: &mut UserEntity) -> ServiceResult<()>;
}

#[async_trait]
pub trait ApubActorRepository {
    async fn get_apub_actor_by_user_id(
        &self,
        user_id: UserID,
    ) -> ServiceResult<Option<ApubActorEntity>>;

    async fn get_apub_actor_by_actor_id(
        &self,
        actor_id: ActorID,
    ) -> ServiceResult<Option<ApubActorEntity>>;
}
