use sea_orm::{ActiveValue::NotSet, Condition, EntityTrait, QueryFilter, Set};

use crate::{
    ServiceResult,
    domain::models::{
        follow::{FollowId, UserFollowEntity},
        user::UserID,
    },
    infrastructures::persistence::sql::LpDbConn,
    repositories::follow::UserFollowRepository,
    services::MapToUnknown,
};
use sea_orm::ActiveModelTrait;
use sea_orm::ColumnTrait;
#[derive(Debug)]
pub struct SqlUserFollowRepository {
    conn: LpDbConn,
}

impl UserFollowRepository for SqlUserFollowRepository {
    async fn save(&self, follow: &mut UserFollowEntity) -> ServiceResult<()> {
        match *follow.id() {
            Some(id) => {
                let model = entity::user_follow::ActiveModel {
                    id: Set(id.as_db()),
                    follower_id: Set(follow.follower_id().as_db()),
                    followed_id: Set(follow.followee_id().as_db()),
                    pending: Set(*follow.pending() as i8),
                    url: Set(follow.url().clone()),
                    created_at: Set(follow.created_at().naive_utc()),
                };
                model.update(self.conn.db()).await.map_err_unknown()?;
                Ok(())
            }
            None => {
                let model = entity::user_follow::ActiveModel {
                    id: NotSet,
                    follower_id: Set(follow.follower_id().as_db()),
                    followed_id: Set(follow.followee_id().as_db()),
                    pending: Set(*follow.pending() as i8),
                    url: Set(follow.url().clone()),
                    created_at: Set(follow.created_at().naive_utc()),
                };
                let model = model.insert(self.conn.db()).await.map_err_unknown()?;
                follow._set_id(FollowId::from_db_trusted(model.id));
                Ok(())
            }
        }
    }

    async fn get_by_follower_and_followee(
        &self,
        follower_id: UserID,
        followee_id: UserID,
    ) -> ServiceResult<Option<UserFollowEntity>> {
        let follow = entity::user_follow::Entity::find()
            .filter(
                Condition::all()
                    .add(entity::user_follow::Column::FollowerId.eq(follower_id.as_db()))
                    .add(entity::user_follow::Column::FollowedId.eq(followee_id.as_db())),
            )
            .one(self.conn.db())
            .await
            .map_err_unknown()?;
        match follow {
            None => Ok(None),
            Some(model) => Ok(Some(UserFollowEntity::new(
                Some(FollowId::from_db_trusted(model.id)),
                UserID::from_db_trusted(model.follower_id),
                UserID::from_db_trusted(model.followed_id),
                model.pending != 0,
                model.url,
                model.created_at.and_utc(),
            ))),
        }
    }
}
