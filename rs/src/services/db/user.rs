use sqlx::prelude::*;
use sqlx::MySqlPool;
use thiserror::Error;
use uuid::fmt::Simple;
use uuid::Uuid;

use crate::models;
use crate::services::LocalUserFindError;
use crate::services::LocalUserFinderService;
use crate::services::ServiceError;
use crate::services::UserAuthService;
use crate::services::UserCreateError;
use crate::services::UserCreateRequest;
use crate::services::UserCreateResult;
use crate::services::UserCreateService;
use crate::services::UserLoginError;
use crate::services::UserLoginRequest;
use crate::services::UserLoginResult;
use crate::utils::generate_uuid;
use crate::utils::user::UserSpecifier;

#[derive(Debug)]
pub struct DBUserCreateService {
    pool: MySqlPool,
}

impl DBUserCreateService {
    pub fn new(pool: MySqlPool) -> Self {
        Self { pool }
    }
}

#[derive(Debug)]
pub struct DBAuthService {
    pool: MySqlPool,
}

impl DBAuthService {
    pub fn new(pool: MySqlPool) -> Self {
        Self { pool }
    }
}

struct LoginDB {
    id: Simple,
    bpasswd: Option<String>,
}

impl UserCreateService for DBUserCreateService {
    async fn create_user(
        &mut self,
        req: &UserCreateRequest,
    ) -> Result<UserCreateResult, ServiceError<UserCreateError>> {
        let user_id = Uuid::new_v4().simple();

        // bcrypt
        let hashed = bcrypt::hash(req.password.clone(), bcrypt::DEFAULT_COST).unwrap();

        sqlx::query!(
            "INSERT INTO users (id, username, nickname, bpasswd) VALUES(?, ?, ?, ?)",
            user_id,
            req.username,
            req.nickname,
            hashed,
        )
        .execute(&self.pool)
        .await?;

        // TODO: conflict handlign

        Ok(UserCreateResult { user_id })
    }

    async fn login_user(
        &mut self,
        req: &UserLoginRequest,
    ) -> Result<UserLoginResult, ServiceError<UserLoginError>> {
        let user = sqlx::query_as!(
            LoginDB,
            "SELECT id AS `id!: Simple`, bpasswd FROM users WHERE username = ? AND host IS NULL",
            &req.username
        )
        .fetch_one(&self.pool)
        .await?;

        if let Some(bpasswd) = user.bpasswd {
            if bcrypt::verify(req.password.clone(), &bpasswd).unwrap() {
                let token = generate_uuid();
                sqlx::query!(
                    "INSERT INTO user_tokens (user_id, token) VALUES(?, ?)",
                    user.id,
                    token
                )
                .execute(&self.pool)
                .await?;
                return Ok(UserLoginResult { user_token: token });
            }
            return Err(ServiceError::SpecificError(UserLoginError::AuthFailed));
        } else {
            return Err(ServiceError::SpecificError(UserLoginError::AuthFailed));
        }
    }
}

impl UserAuthService for DBAuthService {
    async fn authenticate_user(
        &mut self,
        token: &str,
    ) -> Result<models::User, ServiceError<crate::services::AuthError>> {
        let u = sqlx::query_as!(models::User,
            "SELECT users.id AS `id!: Simple`, username, host, nickname, bio, uri, shared_inbox, inbox, outbox, public_key, users.created_at FROM users INNER JOIN user_tokens ON users.id = user_tokens.user_id WHERE token = ?",
            token
        ).fetch_one(&self.pool).await?;

        Ok(u)
    }
}

#[derive(Debug)]
pub struct DBLocalUserFinderService {
    pool: MySqlPool,
}

impl DBLocalUserFinderService {
    pub fn new(pool: MySqlPool) -> Self {
        Self { pool }
    }
}

impl LocalUserFinderService for DBLocalUserFinderService {
    async fn find_user_by_specifier(
        &mut self,
        spec: &UserSpecifier,
    ) -> Result<models::User, ServiceError<LocalUserFindError>> {
        match spec {
            UserSpecifier::Username(username, host) => {
                if let Some(_) = host {
                    return Err(ServiceError::from_se(
                        LocalUserFindError::NotLocalUser.into(),
                    ));
                }

                let u = sqlx::query_as!(models::User,
                "SELECT id AS `id!: Simple`, username, host, nickname, bio, uri, shared_inbox, inbox, outbox, public_key, created_at FROM users WHERE username = ? AND host IS NULL", username).fetch_one(&self.pool).await?;
                return Ok(u);
            }
            UserSpecifier::URL(url) => {
                // check if url is remote
                todo!()
            }
            UserSpecifier::ID(id) => {
                let u = sqlx::query_as!(models::User,
                "SELECT id AS `id!: Simple`, username, host, nickname, bio, uri, shared_inbox, inbox, outbox, public_key, created_at FROM users WHERE id = ?", id.simple().to_string()).fetch_one(&self.pool).await?;
                if u.host.is_some() {
                    return Err(ServiceError::from_se(
                        LocalUserFindError::NotLocalUser.into(),
                    ));
                }
                return Ok(u);
            }
        }
    }
}
