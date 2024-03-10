use sqlx::MySqlPool;
use tracing::{debug, warn};
use uuid::fmt::Simple;

use crate::{
    models,
    services::{
        id::IDGetterService, AllUserFinderService, ApubFollowService, ApubRequestService,
        FollowError, FollowRequestAccepted, FollowRequestSpecifier, ServiceError, SignerError,
        SignerService, UserFindError, UserFollowService,
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
    id_getter: IDGetterService,
}

impl<F: AllUserFinderService, AF: ApubFollowService, R, S> DBUserFollowService<F, AF, R, S> {
    pub fn new(
        pool: MySqlPool,
        finder: F,
        pubfollow: AF,
        req: R,
        signfetch: S,
        id_getter: IDGetterService,
    ) -> Self {
        Self {
            pool,
            finder,
            pubfollow,
            req,
            signfetch,
            id_getter,
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
                    activity,
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

    async fn follow_request_accepted(
        &mut self,
        accepted_request: &crate::services::FollowRequestSpecifier,
    ) -> Result<FollowRequestAccepted, ServiceError<FollowError>> {
        let mut tx = self.pool.begin().await?;

        let follow_req = match accepted_request {
            FollowRequestSpecifier::LocalURI(uri) => {
                debug!("follow request uri: {}", uri);
                let id = self.id_getter.extract_follow_request_id(uri).ok_or_else(|| {
                    ServiceError::from_se(FollowError::RequestNotFound)
                })?;
                debug!("follow request id: {}", id);
                sqlx::query_as!(UserFollowRequest, r#"
                SELECT id AS `id: Simple`, uri, incoming AS `incoming: bool`, follower_id AS `follower_id: Simple`, followee_id AS `followee_id: Simple` FROM user_follow_requests WHERE id = ?
                "#, id).fetch_optional(&mut *tx).await?
            }
            FollowRequestSpecifier::ActorPair(follower_uri, followee_uri) => {
                let follower = self.finder.find_user_by_specifier(follower_uri).await.unwrap();
                let followee = self.finder.find_user_by_specifier(followee_uri).await.unwrap();
                let follower_id = follower.id;
                let followee_id = followee.id;
                sqlx::query_as!(UserFollowRequest, r#"
                SELECT id AS `id: Simple`, uri, incoming AS `incoming: bool`, follower_id AS `follower_id: Simple`, followee_id AS `followee_id: Simple` FROM user_follow_requests WHERE follower_id = ? AND followee_id = ?
                "#, follower_id.to_string(), followee_id.to_string()).fetch_optional(&mut *tx).await?
            }
        }.ok_or_else(|| ServiceError::from_se(FollowError::RequestNotFound))?;

        // insert into actual follow table
        sqlx::query!(
            r#"
            INSERT INTO user_follows (follower_id, followee_id) VALUES (?, ?)
            ON DUPLICATE KEY UPDATE id=id
            "#,
            follow_req.follower_id.to_string(),
            follow_req.followee_id.to_string()
        )
        .execute(&mut *tx)
        .await?;

        // delete the request
        sqlx::query!(
            r#"
            DELETE FROM user_follow_requests WHERE id = ?
            "#,
            follow_req.id.to_string()
        )
        .execute(&mut *tx)
        .await?;

        tx.commit().await?;

        Ok(FollowRequestAccepted {
            follow_req_id: follow_req.id.into(),
            follower_id: follow_req.follower_id.into(),
            followee_id: follow_req.followee_id.into(),
        })
    }
}

#[derive(Debug)]
struct UserFollowRequest {
    id: Simple,
    uri: Option<String>,
    incoming: bool,
    follower_id: Simple,
    followee_id: Simple,
}
