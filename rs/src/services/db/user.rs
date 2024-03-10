use activitystreams::actor::properties::ApActorProperties;
use rsa::pkcs8::DecodePrivateKey;
use rsa::RsaPrivateKey;
use sqlx::MySqlPool;
use tracing::debug;
use uuid::fmt::Simple;
use uuid::Uuid;

use crate::models;
use crate::models::ApubSigner;
use crate::services::id::IDGetterService;
use crate::services::id::UserAttribute;
use crate::services::AllUserFinderService;
use crate::services::ApubRequestService;
use crate::services::LocalUserFindError;
use crate::services::LocalUserFinderService;
use crate::services::ServiceError;
use crate::services::SignerError;
use crate::services::SignerService;
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
use crate::utils::generate_uuid_random;
use crate::utils::user::UserSpecifier;
use rsa::pkcs8::{EncodePrivateKey, EncodePublicKey};
use std::str::FromStr;

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
                let token = generate_uuid_random();
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
                return Err(ServiceError::from_se(LocalUserFindError::NotLocalUser));
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
    id_getter: IDGetterService,
}

impl<R, F> DBAllUserFinderService<R, F> {
    pub fn new(pool: MySqlPool, req: R, local: F, id_getter: IDGetterService) -> Self {
        Self {
            pool,
            req,
            local,
            id_getter,
        }
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

#[derive(Debug)]
struct RemoteUser {
    user_id: Simple,
    following: Option<String>,
    followers: Option<String>,
    liked: Option<String>,
    fetched_at: chrono::NaiveDateTime,
}

async fn find_user_by_url(
    url: impl Into<String>,
    req: &mut impl ApubRequestService,
    pool: &MySqlPool,
) -> Result<models::User, ServiceError<crate::services::UserFindError>> {
    let url = url.into();
    // we can assume that the url is a remote-user url

    // try to fetch user from db
    let u = sqlx::query_as!(models::User,
        "SELECT id AS `id!: Simple`, username, host, nickname, bio, uri, shared_inbox, inbox, outbox, public_key, created_at FROM users WHERE uri = ?", url).fetch_optional(pool).await?;
    let u = if let Some(u) = u {
        // try to fetch remote_users
        let ru = sqlx::query_as!(RemoteUser,
            "SELECT user_id AS `user_id!: Simple`, following, followers, liked, fetched_at FROM remote_users WHERE user_id = ?", u.id.to_string()).fetch_optional(pool).await?;
        if let Some(ru) = ru {
            // check if info is up-to-date
            let now = chrono::Utc::now().naive_utc();
            if now - ru.fetched_at < chrono::Duration::hours(1) {
                return Ok(u);
            }
        }
        Some(u)
    } else {
        None
    };

    // data is not up-to-date, fetch from remote

    let parsed_url = url.parse::<reqwest::Url>().unwrap();
    let host = parsed_url.host_str().unwrap().to_string();

    let actor = req.fetch_user(parsed_url).await.unwrap();

    let username = actor.base.extension.get_preferred_username().unwrap();
    let nickname = actor.as_ref().get_name_xsd_string().unwrap_or(&username);
    let uri = actor.as_ref().get_id().expect("id not set").to_string();
    let shared_inbox = actor
        .base
        .extension
        .get_endpoints()
        .map(|e| e.get_shared_inbox())
        .flatten()
        .map(|s| s.to_string());
    let inbox = actor.base.extension.get_inbox().to_string();
    let outbox = actor.base.extension.get_outbox().to_string();
    let following = actor.base.extension.get_following().map(|s| s.to_string());
    let followers = actor.base.extension.get_followers().map(|s| s.to_string());
    let liked = actor.base.extension.get_liked().map(|s| s.to_string());
    let created_at = chrono::Utc::now().naive_utc();

    let user_id = u.map(|u| u.id).unwrap_or_else(|| generate_uuid());
    let user = models::User {
        id: user_id,
        username: username.to_string(),
        host: Some(host),
        nickname: nickname.to_string(),
        bio: "".to_string(),
        uri: uri.into(),
        shared_inbox,
        inbox: inbox.into(),
        outbox: outbox.into(),
        public_key: None,
        created_at,
    };
    let ru = RemoteUser {
        user_id: user_id,
        following: following,
        followers: followers,
        liked: liked,
        fetched_at: created_at,
    };

    let mut tx = pool.begin().await?;
    sqlx::query!(
        "INSERT INTO users (id, username, host, bpasswd, nickname, uri, shared_inbox, inbox, outbox) VALUES (?,?,?,NULL,?,?,?,?,?) ON DUPLICATE KEY UPDATE username = ?, host = ?, bpasswd = NULL, nickname = ?, uri = ?, shared_inbox = ?, inbox = ?, outbox = ?",
        user.id.to_string(),
        user.username,
        user.host,
        user.nickname,
        user.uri,
        user.shared_inbox,
        user.inbox,
        user.outbox,
        user.username,
        user.host,
        user.nickname,
        user.uri,
        user.shared_inbox,
        user.inbox,
        user.outbox,
    ).execute(&mut *tx).await?;

    sqlx::query!(
        "INSERT INTO remote_users(user_id, following, followers, liked, fetched_at) VALUES(?,?,?,?,?) ON DUPLICATE KEY UPDATE following = ?, followers = ?, liked = ?, fetched_at = ?",
        ru.user_id.to_string(),
        ru.following,
        ru.followers,
        ru.liked,
        ru.fetched_at,
        ru.following,
        ru.followers,
        ru.liked,
        ru.fetched_at,
    ).execute(&mut *tx).await?;

    tx.commit().await?;

    return Ok(user);
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
                    find_user_by_url(wf.api_url(), &mut self.req, &self.pool).await
                } else {
                    self.local
                        .find_user_by_specifier(spec)
                        .await
                        .map_err(map_to_local_error)
                }
            }
            UserSpecifier::URL(api_url) => {
                let local_user_id = self.id_getter.extract_local_user_id(api_url);
                if let Some(local_user_id) = local_user_id {
                    let id = uuid::Uuid::from_str(&local_user_id)
                        .map_err(|_e| ServiceError::SpecificError(UserFindError::UserNotFound))?;
                    self.local
                        .find_user_by_specifier(&UserSpecifier::ID(id))
                        .await
                        .map_err(map_to_local_error)
                } else {
                    find_user_by_url(api_url, &mut self.req, &self.pool).await
                }
            }
            UserSpecifier::ID(_) => self
                .local
                .find_user_by_specifier(spec)
                .await
                .map_err(map_to_local_error),
        }
    }
}

#[derive(Debug)]
pub struct DBSignerService<F> {
    pool: MySqlPool,
    fetch: F,
    id_getter: IDGetterService,
}

pub fn new_db_user_signer_service<F>(
    pool: MySqlPool,
    fetch: F,
    id_getter: IDGetterService,
) -> DBSignerService<F> {
    DBSignerService {
        pool,
        fetch,
        id_getter,
    }
}

#[derive(Debug)]
pub struct DBSigner {
    user_id: String,
    private_key: RsaPrivateKey,
    private_key_id: String,
}

impl ApubSigner for DBSigner {
    fn get_user_id(&self) -> String {
        self.user_id.clone()
    }

    fn get_private_key(&self) -> RsaPrivateKey {
        self.private_key.clone()
    }

    fn get_private_key_id(&self) -> String {
        self.private_key_id.clone()
    }
}

impl<F: LocalUserFinderService> SignerService for DBSignerService<F> {
    type Signer = DBSigner;

    async fn fetch_signer(
        &mut self,
        user: &UserSpecifier,
    ) -> Result<Self::Signer, ServiceError<crate::services::SignerError>> {
        let u = self
            .fetch
            .find_user_by_specifier(user)
            .await
            .map_err(|e| match e {
                ServiceError::SpecificError(
                    LocalUserFindError::NotLocalUser | LocalUserFindError::UserNotFound,
                ) => ServiceError::from_se(SignerError::UserNotFound),
                _ => e.convert(),
            })?;

        let private_key = sqlx::query!(
            "SELECT private_key FROM users WHERE id = ?",
            u.id.to_string()
        )
        .fetch_one(&self.pool)
        .await?;
        let private_key = private_key
            .private_key
            .ok_or(ServiceError::from_se(SignerError::PrivateKeyNotSet))?;
        let private_key_id = self
            .id_getter
            .get_user_id_attr(&u, UserAttribute::PublicKey)
            .unwrap();

        Ok(DBSigner {
            user_id: u.id.to_string(),
            private_key: RsaPrivateKey::from_pkcs8_pem(private_key.as_ref())
                .map_err(|_| ServiceError::from_se(SignerError::PrivateKeyNotSet))?,
            private_key_id,
        })
    }
}
