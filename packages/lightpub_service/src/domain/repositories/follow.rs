use async_trait::async_trait;

use crate::{
    ServiceResult,
    domain::models::{follow::UserFollowEntity, user::UserID},
};

#[async_trait]
pub trait UserFollowRepository {
    async fn save(&self, follow: &mut UserFollowEntity) -> ServiceResult<()>;
    async fn get_by_follower_and_followee(
        &self,
        follower_id: UserID,
        followee_id: UserID,
    ) -> ServiceResult<Option<UserFollowEntity>>;
}
