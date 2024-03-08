use sqlx::MySqlPool;
use tracing::warn;

use crate::{
    models::{self, ApubActivity, ApubPayload, ApubPayloadBuilder},
    services::{
        AllUserFinderService, ApubFollowService, ApubRequestService, FollowError, ServiceError,
        SignerError, SignerService, UserFindError, UserFollowService,
    },
    utils::{generate_uuid, user::UserSpecifier},
};

#[derive(Debug, Clone)]
pub struct DBUserFollowService<F, AF, R, S> {
    pool: MySqlPool,
    finder: F,
    pubfollow: AF,
    req: R,
    signfetch: S,
}

impl<F: AllUserFinderService, AF: ApubFollowService, R, S> DBUserFollowService<F, AF, R, S> {
    pub fn new(pool: MySqlPool, finder: F, pubfollow: AF, req: R, signfetch: S) -> Self {
        Self {
            pool,
            finder,
            pubfollow,
            req,
            signfetch,
        }
    }
}

impl<F: AllUserFinderService, AF: ApubFollowService, R: ApubRequestService, S: SignerService>
    DBUserFollowService<F, AF, R, S>
{
    async fn find_user(
        &mut self,
        user: &UserSpecifier,
        not_found_error: FollowError,
    ) -> Result<models::User, ServiceError<FollowError>> {
        self.finder
            .find_user_by_specifier(user)
            .await
            .map_err(|e| match e {
                ServiceError::SpecificError(UserFindError::UserNotFound) => {
                    ServiceError::from_se(not_found_error)
                }
                _ => e.convert(),
            })
    }
}

impl<F: AllUserFinderService, AF: ApubFollowService, R: ApubRequestService, S: SignerService>
    UserFollowService for DBUserFollowService<F, AF, R, S>
{
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

        // follower must be a local user
        assert!(follower.host.is_none());

        let follower_id = &follower.id;
        let followee_id = &followee.id;

        if let Some(_) = followee.host {
            let followee_inbox = followee.inbox.clone().ok_or_else(|| {
                warn!("followee inbox not set: {:?}", followee);
                ServiceError::from_se(FollowError::FolloweeNotFound)
            })?;
            let request_id = generate_uuid();
            sqlx::query!(
                r#"
                INSERT INTO user_follow_requests (id, incoming, follower_id, followee_id) VALUES (?, ?, ?, ?)
                "#,
                request_id.to_string(),
                0,
                follower_id.to_string(),
                followee_id.to_string()
            ).execute(&self.pool).await?;

            // send request
            let actor = self
                .signfetch
                .fetch_signer(&UserSpecifier::from_id(follower.id))
                .await
                .map_err(|e| match e {
                    ServiceError::SpecificError(SignerError::UserNotFound) => {
                        ServiceError::from_se(FollowError::FollowerNotFound)
                    }
                    _ => e.convert(),
                })?;
            let activity = self
                .pubfollow
                .create_follow_request(request_id.into())
                .await
                .unwrap();
            self.req
                .post_to_inbox(
                    followee_inbox.parse::<reqwest::Url>().unwrap(),
                    &ApubPayloadBuilder::new(ApubActivity::Follow(activity))
                        .with_context("https://www.w3.org/ns/activitystreams")
                        .build(),
                    &actor,
                )
                .await
                .map_err(|e| e.convert())?;
        } else {
            // local follow
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
        }

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
