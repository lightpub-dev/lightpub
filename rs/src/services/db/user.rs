use sqlx::MySqlPool;
use uuid::fmt::Simple;
use uuid::Uuid;

use crate::models;
use crate::services::AllUserFinderService;
use crate::services::ApubRequestService;
use crate::services::LocalUserFindError;
use crate::services::LocalUserFinderService;
use crate::services::ServiceError;
use crate::services::UserAuthService;
use crate::services::UserCreateError;
use crate::services::UserCreateRequest;
use crate::services::UserCreateResult;
use crate::services::UserCreateService;
use crate::services::UserFindError;
use crate::services::UserLoginError;
use crate::services::UserLoginRequest;
use crate::services::UserLoginResult;
use crate::utils;
use crate::utils::generate_uuid;
use crate::utils::user::UserSpecifier;
use rsa::pkcs8::{EncodePrivateKey, EncodePublicKey};

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

        // generate rsa keys
        let (private_key, public_key) = tokio::task::spawn_blocking(|| {
            let private_key = utils::key::generate();
            let public_key = private_key.to_public_key();
            let private_key = private_key
                .to_pkcs8_pem(pkcs8::LineEnding::LF)
                .unwrap()
                .to_string();
            let public_key = public_key.to_public_key_pem(pkcs8::LineEnding::LF).unwrap();
            (private_key, public_key)
        })
        .await
        .unwrap();

        sqlx::query!(
            "INSERT INTO users (id, username, nickname, bpasswd, private_key, public_key) VALUES(?, ?, ?, ?, ?, ?)",
            user_id,
            req.username,
            req.nickname,
            hashed,
            private_key,
            public_key
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
            UserSpecifier::URL(_url) => {
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

#[derive(Debug)]
pub struct DBAllUserFinderService<R, F> {
    pool: MySqlPool,
    req: R,
    local: F,
}

impl<R, F> DBAllUserFinderService<R, F> {
    pub fn new(pool: MySqlPool, req: R, local: F) -> Self {
        Self { pool, req, local }
    }
}

fn map_to_local_error(e: ServiceError<LocalUserFindError>) -> ServiceError<UserFindError> {
    match e {
        ServiceError::SpecificError(e) => match e {
            LocalUserFindError::NotLocalUser => {
                panic!("should not happen")
            }
            LocalUserFindError::UserNotFound => ServiceError::from_se(UserFindError::UserNotFound),
        },
        _ => e.convert(),
    }
}

impl<R: ApubRequestService, F: LocalUserFinderService> AllUserFinderService
    for DBAllUserFinderService<R, F>
{
    async fn find_user_by_specifier(
        &mut self,
        spec: &UserSpecifier,
    ) -> Result<models::User, ServiceError<crate::services::UserFindError>> {
        match spec {
            UserSpecifier::Username(username, host) => {
                if let Some(host) = host {
                    // check if already exists in local db
                    let u = sqlx::query_as!(models::User,
                        "SELECT id AS `id!: Simple`, username, host, nickname, bio, uri, shared_inbox, inbox, outbox, public_key, created_at FROM users WHERE username = ? AND host = ?", username, host).fetch_optional(&self.pool).await?;
                    if let Some(u) = u {
                        // TODO: check if user is up-to-date
                        return Ok(u);
                    }

                    let wf = self
                        .req
                        .fetch_webfinger(username, host)
                        .await
                        .map_err(|e| e.convert())?;
                    let api_url = wf.api_url();
                    let actor = self
                        .req
                        .fetch_user(api_url.parse::<reqwest::Url>().unwrap())
                        .await
                        .unwrap();

                    let user_id = generate_uuid();
                    let user = models::User {
                        id: user_id,
                        username: actor
                            .preferred_username()
                            .clone()
                            .ok_or(ServiceError::from_se(UserFindError::RemoteError))?,
                        host: Some(host.clone()),
                        nickname: actor.name().clone().unwrap_or({
                            actor
                                .preferred_username()
                                .clone()
                                .ok_or(ServiceError::from_se(UserFindError::RemoteError))?
                        }),
                        bio: "".to_string(),
                        uri: Some(actor.id().to_owned()),
                        shared_inbox: actor.shared_inbox().to_owned(),
                        inbox: Some(actor.inbox().to_owned()),
                        outbox: Some(actor.outbox().to_owned()),
                        public_key: None,
                        created_at: chrono::Utc::now().naive_utc(),
                    };

                    sqlx::query!(
                        "INSERT INTO users (id, username, host, bpasswd, nickname, uri, shared_inbox, inbox, outbox) VALUES (?,?,?,NULL,?,?,?,?,?)",
                        user.id.to_string(),
                        user.username,
                        user.host,
                        user.nickname,
                        user.uri,
                        user.shared_inbox,
                        user.inbox,
                        user.outbox
                    ).execute(&self.pool).await?;

                    return Ok(user);
                } else {
                    self.local
                        .find_user_by_specifier(spec)
                        .await
                        .map_err(map_to_local_error)
                }
            }
            UserSpecifier::URL(_url) => {
                // check if url is remote
                todo!()
            }
            UserSpecifier::ID(id) => self
                .local
                .find_user_by_specifier(spec)
                .await
                .map_err(map_to_local_error),
        }
    }
}
