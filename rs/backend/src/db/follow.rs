use async_trait::async_trait;
use sqlx::MySqlPool;
use tracing::{debug, warn};
use uuid::fmt::Simple;

use crate::{
    holder, id::IDGetterService, AllUserFinderService, ApubFollowService, ApubRequestService,
    FetchFollowListOptions, FollowError, FollowRequestAccepted, FollowRequestSpecifier, Holder,
    IncomingFollowRequest, ServiceError, SignerError, SignerService, UserFindError,
    UserFollowService,
};
use lightpub_model::{
    self,
    api_response::{FollowListEntry, FollowListEntryBuilder},
    HasRemoteUri, User, UserSpecifier,
};
use lightpub_utils::generate_uuid;

pub struct DBUserFollowService {
    pool: MySqlPool,
    finder: holder!(AllUserFinderService),
    pubfollow: holder!(ApubFollowService),
    req: holder!(ApubRequestService),
    signfetch: holder!(SignerService),
    id_getter: IDGetterService,
}

impl DBUserFollowService {
    pub fn new(
        pool: MySqlPool,
        finder: holder!(AllUserFinderService),
        pubfollow: holder!(ApubFollowService),
        req: holder!(ApubRequestService),
        signfetch: holder!(SignerService),
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

impl DBUserFollowService {
    async fn find_user(
        &mut self,
        user: &UserSpecifier,
        not_found_error: FollowError,
    ) -> Result<User, ServiceError<FollowError>> {
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

#[derive(Debug)]
struct FollowUser {
    id: Simple,
    uri: Option<String>,
    username: String,
    host: Option<String>,
    avatar_id: Option<Simple>,
    nickname: String,
    created_at: chrono::NaiveDateTime,
}

impl HasRemoteUri for FollowUser {
    fn get_local_id(&self) -> String {
        self.id.to_string()
    }

    fn get_remote_uri(&self) -> Option<String> {
        self.uri.clone()
    }
}

#[async_trait]
impl UserFollowService for DBUserFollowService {
    async fn is_following(
        &mut self,
        follower: &UserSpecifier,
        followee: &UserSpecifier,
    ) -> Result<bool, anyhow::Error> {
        let follower = self
            .find_user(follower, FollowError::FollowerNotFound)
            .await?;
        let followee = self
            .find_user(followee, FollowError::FolloweeNotFound)
            .await?;

        let follower_id = &follower.id;
        let followee_id = &followee.id;

        let row = sqlx::query!(
            r#"
            SELECT id FROM user_follows WHERE follower_id=? AND followee_id=? LIMIT 1
            "#,
            follower_id.to_string(),
            followee_id.to_string()
        )
        .fetch_optional(&self.pool)
        .await?;

        Ok(row.is_some())
    }

    async fn fetch_following_list(
        &mut self,
        user: &UserSpecifier,
        options: &FetchFollowListOptions,
    ) -> Result<Vec<FollowListEntry>, anyhow::Error> {
        let user = self.find_user(user, FollowError::FollowerNotFound).await?;
        let (before_date_valid, before_date) = match options.before_date {
            Some(date) => (true, Some(date.naive_utc())),
            None => (false, None),
        };

        let rows = sqlx::query_as!(FollowUser,
        r#"
            SELECT u.id AS `id: Simple`, u.uri, u.username, u.host, u.avatar_id AS `avatar_id: Simple`, u.nickname, f.created_at
            FROM users u
            INNER JOIN user_follows f ON u.id = f.followee_id
            WHERE f.follower_id = ? AND (NOT ? OR f.created_at <= ?)
            ORDER BY f.created_at DESC
            LIMIT ?
        "#, user.id.to_string(), before_date_valid, before_date, options.limit)
            .fetch_all(&self.pool)
            .await?;

        Ok(rows
            .into_iter()
            .map(|row| {
                let uri = self.id_getter.get_user_id(&row);

                FollowListEntryBuilder::default()
                    .id(row.id.into())
                    .uri(uri)
                    .username(row.username)
                    .host(row.host)
                    .avatar_id(row.avatar_id.map(|id| id.into()))
                    .nickname(row.nickname)
                    .created_at(chrono::DateTime::from_naive_utc_and_offset(
                        row.created_at,
                        chrono::Utc,
                    ))
                    .build()
                    .unwrap()
            })
            .collect())
    }

    async fn fetch_follower_list(
        &mut self,
        user: &UserSpecifier,
        options: &FetchFollowListOptions,
    ) -> Result<Vec<FollowListEntry>, anyhow::Error> {
        let user = self.find_user(user, FollowError::FollowerNotFound).await?;
        let (before_date_valid, before_date) = match options.before_date {
            Some(date) => (true, Some(date.naive_utc())),
            None => (false, None),
        };

        let rows = sqlx::query_as!(FollowUser,
        r#"
            SELECT u.id AS `id: Simple`, u.uri, u.username, u.host, u.avatar_id AS `avatar_id: Simple`, u.nickname, f.created_at
            FROM users u
            INNER JOIN user_follows f ON u.id = f.follower_id
            WHERE f.followee_id = ? AND (NOT ? OR f.created_at <= ?)
            ORDER BY f.created_at DESC
            LIMIT ?
        "#,
        user.id.to_string(),
        before_date_valid,
        before_date,
        options.limit)
            .fetch_all(&self.pool)
            .await?;

        Ok(rows
            .into_iter()
            .map(|row| {
                let uri = self.id_getter.get_user_id(&row);

                FollowListEntryBuilder::default()
                    .id(row.id.into())
                    .uri(uri)
                    .username(row.username)
                    .host(row.host)
                    .avatar_id(row.avatar_id.map(|id| id.into()))
                    .nickname(row.nickname)
                    .created_at(chrono::DateTime::from_naive_utc_and_offset(
                        row.created_at,
                        chrono::Utc,
                    ))
                    .build()
                    .unwrap()
            })
            .collect())
    }

    async fn follow_user(
        &mut self,
        follower_spec: &UserSpecifier,
        followee_spec: &UserSpecifier,
    ) -> Result<(), crate::ServiceError<crate::FollowError>> {
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
                .post_to_inbox(&followee_inbox, &activity.into(), actor)
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
    ) -> Result<(), crate::ServiceError<crate::FollowError>> {
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

        // remote unfollow
        if let Some(_) = followee.host {
            let followee_inbox = followee.inbox.clone().ok_or_else(|| {
                warn!("followee inbox not set: {:?}", followee);
                ServiceError::from_se(FollowError::FolloweeNotFound)
            })?;

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
                .create_unfollow_request(
                    &UserSpecifier::from_id(*follower_id),
                    &UserSpecifier::from_id(*followee_id),
                )
                .await
                .unwrap();

            self.req
                .post_to_inbox(&followee_inbox, &activity.into(), actor)
                .await
                .map_err(|e| e.convert())?;
        }

        Ok(())
    }

    async fn follow_request_accepted(
        &mut self,
        accepted_request: &crate::FollowRequestSpecifier,
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

    async fn incoming_follow_request(
        &mut self,
        incoming_follow_request: &IncomingFollowRequest,
    ) -> Result<(), ServiceError<FollowError>> {
        let (req_uri, follower_uri, followee_uri) = match incoming_follow_request {
            IncomingFollowRequest::ActorPair(uri, f1, f2) => (uri, f1, f2),
        };

        let follower = self
            .finder
            .find_user_by_specifier(follower_uri)
            .await
            .or(Err(ServiceError::from_se(FollowError::FollowerNotFound)))?;
        let followee = self
            .finder
            .find_user_by_specifier(followee_uri)
            .await
            .or(Err(ServiceError::from_se(FollowError::FolloweeNotFound)))?;

        let follow_req_id = generate_uuid();
        let follow_req_uri = req_uri;
        let follower_id = follower.id;
        let followee_id = followee.id;
        sqlx::query!(
            "INSERT INTO user_follow_requests (id, uri, incoming, follower_id, followee_id) VALUES (?, ?, 1, ?, ?) ON DUPLICATE KEY UPDATE id=?, uri=?",
            follow_req_id.to_string(),
            follow_req_uri,
            follower_id.to_string(),
            followee_id.to_string(),
            follow_req_id.to_string(),
            follow_req_uri,
        ).execute(&self.pool).await?;

        // TODO: execute codes below in the background runner
        {
            let follower_inbox = follower.inbox.clone().ok_or_else(|| {
                warn!("follower inbox not set: {:?}", follower);
                ServiceError::from_se(FollowError::FollowerNotFound)
            })?;
            let actor = self
                .signfetch
                .fetch_signer(&UserSpecifier::from_id(followee.id))
                .await
                .map_err(|e| match e {
                    ServiceError::SpecificError(SignerError::UserNotFound) => {
                        ServiceError::from_se(FollowError::FolloweeNotFound)
                    }
                    _ => e.convert(),
                })?;
            // debug!("create_follow_accept({:?})", follow_req_id);
            let activity = self
                .pubfollow
                .create_follow_accept(follow_req_id.into())
                .await
                .unwrap();

            self.req
                .post_to_inbox(&follower_inbox, &activity.into(), actor)
                .await
                .map_err(|e| e.convert())?;

            // delete from user_follow_requests
            // and insert actual follow
            let mut tx = self.pool.begin().await?;

            sqlx::query!(
                r#"
                DELETE FROM user_follow_requests WHERE id = ?
                "#,
                follow_req_id.to_string()
            )
            .execute(&mut *tx)
            .await?;

            sqlx::query!(
                r#"
                INSERT INTO user_follows (follower_id, followee_id) VALUES (?, ?)
                ON DUPLICATE KEY UPDATE id=id
                "#,
                follower_id.to_string(),
                followee_id.to_string()
            )
            .execute(&mut *tx)
            .await?;

            tx.commit().await?;
        }

        Ok(())
    }
}

#[derive(Debug)]
struct UserFollowRequest {
    id: Simple,
    #[allow(dead_code)]
    uri: Option<String>,
    #[allow(dead_code)]
    incoming: bool,
    follower_id: Simple,
    followee_id: Simple,
}
