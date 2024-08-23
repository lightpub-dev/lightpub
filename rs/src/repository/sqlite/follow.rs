use async_trait::async_trait;

use crate::{
    domain::model::{follow::UserFollow, user::UserId},
    repository::interface::{follow::FollowRepository, RepositoryError},
};

use super::SqliteRepository;

#[async_trait]
impl<'a> FollowRepository for SqliteRepository<'a> {
    async fn follow(&mut self, follow: &UserFollow) -> Result<UserId, RepositoryError> {
        todo!()
    }

    async fn unfollow(&mut self, follow: &UserFollow) -> Result<(), RepositoryError> {
        todo!()
    }
    async fn find_by_user_id(
        &mut self,
        follower_id: &UserId,
        followee_id: &UserId,
    ) -> Result<Option<UserFollow>, RepositoryError> {
        sqlx::query!(r#"SELECT * FROM users"#)
            .fetch_optional(self)
            .await
            .unwrap();

        todo!()
    }
}
