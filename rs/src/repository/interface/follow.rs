use async_trait::async_trait;

use crate::domain::model::{
    follow::{FollowId, UserFollow},
    user::{User, UserId, Username},
};

use super::RepositoryError;

#[async_trait]
pub trait FollowRepository {
    async fn follow(&mut self, follow: &UserFollow) -> Result<UserId, RepositoryError>;
    async fn unfollow(&mut self, follow: &UserFollow) -> Result<(), RepositoryError>;
    async fn find_by_user_id(
        &mut self,
        follower_id: &UserId,
        followee_id: &UserId,
    ) -> Result<Option<UserFollow>, RepositoryError>;
}
