use async_trait::async_trait;

use crate::domain::model::{
    follow::{FollowId, UserFollow},
    user::{User, UserId, Username},
};

use super::RepositoryError;

#[async_trait]
pub trait FollowRepository {
    async fn create_if_not_exists(
        &mut self,
        follow: &mut UserFollow, // id field will be set
    ) -> Result<(), RepositoryError>;
    async fn delete_if_exists(&mut self, follow: &UserFollow) -> Result<(), RepositoryError>;
    async fn find_by_user_id(
        &mut self,
        follower_id: &UserId,
        followee_id: &UserId,
    ) -> Result<Option<UserFollow>, RepositoryError>;
}
