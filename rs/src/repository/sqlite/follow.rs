use crate::domain::model::DateTime;
use async_trait::async_trait;

use crate::{
    domain::model::{
        follow::{FollowId, UserFollow},
        user::UserId,
    },
    repository::interface::{follow::FollowRepository, RepositoryError},
};

use super::{IsUuid, SqliteRepository, SqliteUuid};

#[async_trait]
impl<'c, 'tx> FollowRepository for SqliteRepository<'c, 'tx> {
    async fn create_if_not_exists(
        &mut self,
        follow: &mut UserFollow,
    ) -> Result<(), RepositoryError> {
        let follower_id = follow.follower().to_db();
        let followee_id = follow.followee().to_db();
        let follow_on = follow.follow_on();
        let result = sqlx::query!(
            r#"INSERT INTO user_follows(follower_id,followee_id,created_at) VALUES (?,?,?)"#,
            follower_id,
            followee_id,
            follow_on,
        )
        .execute(&mut self.conn)
        .await
        .unwrap();

        follow.set_id(FollowId::from_int(result.last_insert_rowid()));

        Ok(())
    }

    async fn delete_if_exists(&mut self, follow: &UserFollow) -> Result<(), RepositoryError> {
        if let Some(id) = follow.id() {
            sqlx::query!(r#"DELETE FROM user_follows WHERE id=?"#, id)
                .execute(&mut self.conn)
                .await
                .unwrap();

            Ok(())
        } else {
            panic!("id not set");
        }
    }

    async fn find_by_user_id(
        &mut self,
        follower_id: &UserId,
        followee_id: &UserId,
    ) -> Result<Option<UserFollow>, RepositoryError> {
        let follower_uuid = follower_id.to_db();
        let followee_uuid = followee_id.to_db();
        let result = sqlx::query!(r#"SELECT id AS `id: FollowId`,follower_id AS `follower_id: SqliteUuid`,followee_id AS `followee_id: SqliteUuid`,created_at AS `created_at: DateTime` FROM user_follows WHERE follower_id=? AND followee_id=?"#, follower_uuid, followee_uuid)
            .fetch_optional(&mut self.conn)
            .await
            .unwrap();

        Ok(result.map(|row| {
            let mut follow = UserFollow::new(
                row.follower_id.into_domain(),
                row.followee_id.into_domain(),
                row.created_at,
            );
            follow.set_id(row.id);
            follow
        }))
    }
}
