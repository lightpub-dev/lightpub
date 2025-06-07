use crate::{
    ServiceResult,
    domain::models::{
        upload::UploadID,
        user::{Domain, Nickname, UserConfig, UserEntity, UserID, UserProfile, Username},
    },
    infrastructures::persistence::sql::LpKvConn,
    repositories::user::UserRepository,
    services::{MapToUnknown, ServiceError},
};
use sea_orm::{ActiveModelTrait, Set};
use sea_orm::{ColumnTrait, PaginatorTrait};
use sea_orm::{EntityTrait, QueryFilter};

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
            UserConfig::new(
                user.is_bot != 0,
                user.is_admin != 0,
                user.auto_follow_accept != 0,
                user.hide_follows != 0,
            ),
            false, // is_dirty
            true,  // in_db
        );

        self.kv.set(format!("user:{user_id}"), &model).await?;

        Ok(Some(model))
    }

    async fn get_total_users_count(&self) -> ServiceResult<u64> {
        let count = entity::user::Entity::find()
            .filter(entity::user::Column::Domain.eq(""))
            .count(self.db.db())
            .await
            .map_err_unknown()?;

        Ok(count)
    }

    async fn update(&self, user: &mut UserEntity) -> ServiceResult<()> {
        if !*user.is_dirty() {
            return Ok(());
        }
        if !*user.in_db() {
            return Err(ServiceError::ise("user must be in db to update"));
        }

        let model = entity::user::ActiveModel {
            id: Set(user.id().as_db()),
            username: Set(user.username().as_str().to_owned()),
            domain: Set(user.domain().as_db().to_owned()),
            nickname: Set(user.nickname().as_str().to_owned()),
            bio: Set(user.profile().bio().clone()),
            avatar: Set(user.profile().avatar().as_ref().map(|a| a.as_db())),
            is_bot: Set(*user.config().is_bot() as i8),
            is_admin: Set(*user.config().is_admin() as i8),
            auto_follow_accept: Set(*user.config().auto_follow_accept() as i8),
            hide_follows: Set(*user.config().hide_follows() as i8),
            created_at: Set(user.profile().created_at().map(|d| d.naive_utc())),
            ..Default::default()
        };

        model.update(self.db.db()).await.map_err_unknown()?;

        Ok(())
    }
}

impl SqlUserRepository {
    async fn invalidate_user_cache(&self, user_id: UserID) -> ServiceResult<()> {
        self.kv.delete(format!("user:{user_id}")).await?;
        Ok(())
    }
}
