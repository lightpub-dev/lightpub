use derive_more::Constructor;

use crate::{
    ServiceResult,
    applications::dto::user::UserRegisterDto,
    domain::{models::user::UserEntity, repositories::user::UserRepository},
    infrastructures::locator::ServiceLocator,
    services::ServiceError,
};

#[derive(Debug, Constructor)]
pub struct UserRegisterService {
    locator: ServiceLocator,
}

impl UserRegisterService {
    pub async fn register_user(&self, dto: UserRegisterDto) -> ServiceResult<UserEntity> {
        let user_repo = self.locator.user_repository();

        let mut user = UserEntity::create_new(dto.username, dto.nickname);

        user_repo.save(&mut user).await?;

        let new_user = user_repo.get_user_by_id(user.id()).await?;
        let new_user = match new_user {
            Some(user) => user,
            None => return Err(ServiceError::ise("User not found after registration.")),
        };

        let hash_service = self.locator.hash_service();
        let user_auth_repo = self.locator.user_auth_repository();
        let mut user_auth = hash_service.create_user_auth(new_user.id(), &dto.password)?;
        user_auth_repo.save(&mut user_auth).await?;

        Ok(new_user)
    }
}
