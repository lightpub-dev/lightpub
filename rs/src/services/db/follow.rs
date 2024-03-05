use sqlx::MySqlPool;

use crate::{
    models,
    services::{
        FollowError, LocalUserFindError, LocalUserFinderService, ServiceError, UserFollowService,
    },
    utils::user::UserSpecifier,
};

#[derive(Debug, Clone)]
pub struct DBUserFollowService<F> {
    pool: MySqlPool,
    finder: F,
}

impl<F: LocalUserFinderService> DBUserFollowService<F> {
    pub fn new(pool: MySqlPool, finder: F) -> Self {
        Self { pool, finder }
    }
}

impl<F: LocalUserFinderService> DBUserFollowService<F> {
    async fn find_user(
        &mut self,
        user: &UserSpecifier,
        not_found_error: FollowError,
    ) -> Result<models::User, ServiceError<FollowError>> {
        self.finder
            .find_user_by_specifier(user)
            .await
            .map_err(|e| match e {
                ServiceError::SpecificError(LocalUserFindError::UserNotFound) => {
                    ServiceError::from_se(not_found_error)
                }
                _ => e.convert(),
            })
    }
}

impl<F: LocalUserFinderService> UserFollowService for DBUserFollowService<F> {
    async fn follow_user(
        &mut self,
        follower_spec: &UserSpecifier,
        followee_spec: &UserSpecifier,
    ) -> Result<(), crate::services::ServiceError<crate::services::FollowError>> {
        let follower = self
            .find_user(follower_spec, FollowError::FollowerNotFound)
            .await?;
        let followee = self
            .find_user(followee_spec, FollowError::FolloweeNotFound)
            .await?;

        let follower_id = &follower.id;
        let followee_id = &followee.id;

        sqlx::query!(
            r#"
            INSERT INTO user_follows (follower_id, followee_id) VALUES(?,?)
            ON DUPLICATE KEY UPDATE id=id
            "#,
            follower_id.to_string(),
            followee_id.to_string()
        )
        .execute(&self.pool)
        .await?;

        Ok(())
    }

    async fn unfollow_user(
        &mut self,
        follower_spec: &UserSpecifier,
        followee_spec: &UserSpecifier,
    ) -> Result<(), crate::services::ServiceError<crate::services::FollowError>> {
        let follower = self
            .find_user(follower_spec, FollowError::FollowerNotFound)
            .await?;
        let followee = self
            .find_user(followee_spec, FollowError::FolloweeNotFound)
            .await?;

        let follower_id = &follower.id;
        let followee_id = &followee.id;

        sqlx::query!(
            r#"
            DELETE FROM user_follows WHERE follower_id=? AND followee_id=?
            "#,
            follower_id.to_string(),
            followee_id.to_string()
        )
        .execute(&self.pool)
        .await?;

        Ok(())
    }
}
