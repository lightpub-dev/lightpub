use sea_orm::EntityTrait;

use crate::{
    ServiceResult,
    domain::models::{
        upload::UploadID,
        user::{Domain, Nickname, UserConfig, UserEntity, UserID, UserProfile, Username},
    },
    repositories::{sql::LpKvConn, user::UserRepository},
    services::ServiceError,
};

use super::LpDbConn;

pub struct SqlUserRepository {
    db: LpDbConn,
    kv: LpKvConn,
}

impl UserRepository for SqlUserRepository {
    async fn get_user_by_id(&self, user_id: UserID) -> ServiceResult<Option<UserEntity>> {
        let cache = self.kv.get(format!("user:{user_id}")).await?;
        if let Some(cache) = cache {
            return Ok(Some(cache));
        }

        let user = entity::user::Entity::find_by_id(user_id.as_db())
            .one(self.db.db())
            .await
            .map_err(ServiceError::unknown)?;

        let user = match user {
            None => return Ok(None),
            Some(user) => user,
        };

        let avatar = user
            .avatar
            .as_ref()
            .map(|a| UploadID::from_db_trusted(a.clone()));

        let model = UserEntity::new(
            UserID::from_db_trusted(user.id),
            Username::from_str(&user.username),
            Domain::from_str(user.domain.clone()),
            Nickname::from_str(user.nickname),
            UserProfile::new(
                user.bio.clone(),
                avatar,
                user.fetched_at.map(|d| d.and_utc()),
                user.created_at.map(|d| d.and_utc()),
            ),
            UserConfig::new(user.is_bot != 0, user.auto_follow_accept != 0),
        );

        self.kv.set(format!("user:{user_id}"), &model).await?;

        Ok(Some(model))
    }
}
