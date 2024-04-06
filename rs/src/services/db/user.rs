use async_trait::async_trait;
use derive_more::Constructor;
use rsa::pkcs8::DecodePrivateKey;
use rsa::RsaPrivateKey;
use sqlx::MySqlPool;
use tracing::warn;
use uuid::fmt::Simple;
use uuid::Uuid;

use crate::holder;
use crate::models;
use crate::models::apub::Actor;
use crate::models::ApubSigner;
use crate::services::id::IDGetterService;
use crate::services::id::UserAttribute;
use crate::services::AllUserFinderService;
use crate::services::ApubRequestService;
use crate::services::InboxPair;
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
use crate::services::UserProfileService;
use crate::services::UserProfileUpdate;
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

#[async_trait]
impl UserCreateService for DBUserCreateService {
    async fn create_user(
        &mut self,
        req: &UserCreateRequest,
    ) -> Result<UserCreateResult, ServiceError<UserCreateError>> {
        let exist = sqlx::query!(
            r#"
        SELECT COUNT(*) AS count FROM users WHERE username=? AND host IS NULL FOR UPDATE
        "#,
            req.username
        )
        .fetch_one(&self.pool)
        .await?;
        if exist.count > 0 {
            return Err(ServiceError::SpecificError(
                UserCreateError::UsernameConflict,
            ));
        }

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
        .fetch_optional(&self.pool)
        .await?;

        let user = if let Some(u) = user {
            u
        } else {
            return Err(ServiceError::from_se(UserLoginError::AuthFailed));
        };

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

#[async_trait]
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

#[async_trait]
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

pub struct DBAllUserFinderService {
    pool: MySqlPool,
    req: holder!(ApubRequestService),
    local: holder!(LocalUserFinderService),
    id_getter: IDGetterService,
}

impl DBAllUserFinderService {
    pub fn new(
        pool: MySqlPool,
        req: holder!(ApubRequestService),
        local: holder!(LocalUserFinderService),
        id_getter: IDGetterService,
    ) -> Self {
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
    req: &mut holder!(ApubRequestService),
    pool: &MySqlPool,
) -> Result<models::User, ServiceError<crate::services::UserFindError>> {
    let url = url.into();
    // we can assume that the url is a remote-user url

    // try to fetch user from db
    let u = sqlx::query_as!(models::User,
        "SELECT id AS `id!: Simple`, username, host, nickname, bio, uri, shared_inbox, inbox, outbox, public_key, created_at FROM users WHERE uri = ?", url).fetch_optional(pool).await?;
    if let Some(u) = u {
        // try to fetch remote_users
        let ru = sqlx::query_as!(RemoteUser,
            "SELECT user_id AS `user_id!: Simple`, following, followers, liked, fetched_at FROM remote_users WHERE user_id = ?", u.id.to_string()).fetch_optional(pool).await?;
        if let Some(ru) = ru {
            // check if info is up-to-date
            let now = chrono::Utc::now().naive_utc();
            if now - ru.fetched_at < chrono::Duration::try_hours(1).unwrap() {
                return Ok(u);
            }
        }
    };

    // data is not up-to-date, fetch from remote

    let parsed_url = url.parse::<reqwest::Url>().unwrap();
    let host = parsed_url.host_str().unwrap().to_string();

    let actor = req.fetch_user(parsed_url.as_str()).await.unwrap();
    let actor = match actor {
        Actor::Person(a) => a,
        Actor::Application(a) => a,
    };

    let username = actor.preferred_username;
    let nickname = actor.name;
    let uri = actor.id;
    let shared_inbox = actor.shared_inbox;
    let inbox = actor.inbox;
    let outbox = actor.outbox;
    let following = actor.following;
    let followers = actor.followers;
    let liked = actor.liked;
    let pubkey = actor.public_key;
    let pubkey_pem = pubkey.public_key_pem;
    let pubkey_owner = pubkey.owner;
    let pubkey_id = pubkey.id;
    let created_at = chrono::Utc::now().naive_utc();
    let bio = actor.summary.unwrap_or("".to_string());

    if uri != pubkey_owner {
        warn!("pubkey owner and user uri does not match");
        return Err(ServiceError::from_se(UserFindError::RemoteError));
    }

    // refetch user_id
    // (actor.id may be different from the url we fetched from)
    let user_id = sqlx::query!("SELECT id AS `id!: Simple` FROM users WHERE uri = ?", uri)
        .fetch_optional(pool)
        .await?;
    let user_id = if let Some(u) = user_id {
        u.id
    } else {
        // generate new user_id
        generate_uuid()
    };

    let user = models::User {
        id: user_id,
        username: username.to_string(),
        host: Some(host),
        nickname: nickname.to_string(),
        bio: bio,
        uri: uri.into(),
        shared_inbox,
        inbox: inbox.into(),
        outbox: outbox.into(),
        public_key: Some(pubkey_pem.clone()),
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
        "INSERT INTO users (id, username, host, bpasswd, nickname, uri, shared_inbox, inbox, outbox, bio) VALUES (?,?,?,NULL,?,?,?,?,?,?) ON DUPLICATE KEY UPDATE username = ?, host = ?, bpasswd = NULL, nickname = ?, uri = ?, shared_inbox = ?, inbox = ?, outbox = ?, bio = ?",
        user.id.to_string(),
        user.username,
        user.host,
        user.nickname,
        user.uri,
        user.shared_inbox,
        user.inbox,
        user.outbox,
        user.bio,
        user.username,
        user.host,
        user.nickname,
        user.uri,
        user.shared_inbox,
        user.inbox,
        user.outbox,
        user.bio,
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

    sqlx::query!("
        INSERT INTO user_keys (id, owner_id, public_key) VALUES (?,?,?) ON DUPLICATE KEY UPDATE owner_id=?, public_key=?, updated_at=CURRENT_TIMESTAMP
    ", pubkey_id, user_id.to_string(), pubkey_pem, user_id.to_string(), pubkey_pem).execute(&mut *tx).await?;

    tx.commit().await?;

    return Ok(user);
}

#[async_trait]
impl AllUserFinderService for DBAllUserFinderService {
    async fn find_user_by_specifier(
        &mut self,
        spec: &UserSpecifier,
    ) -> Result<models::User, ServiceError<crate::services::UserFindError>> {
        match spec {
            UserSpecifier::Username(username, host) => {
                if let Some(host) = host {
                    if self.id_getter.is_our_host(host) {
                        // local user
                        return self
                            .local
                            .find_user_by_specifier(&UserSpecifier::from_username(
                                username.to_string(),
                                None,
                            ))
                            .await
                            .map_err(map_to_local_error);
                    }

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
            UserSpecifier::ID(id) => {
                let u = sqlx::query_as!(models::User,
                    "SELECT id AS `id!: Simple`, username, host, nickname, bio, uri, shared_inbox, inbox, outbox, public_key, created_at FROM users WHERE id = ?", id.simple().to_string()).fetch_one(&self.pool).await?;

                return Ok(u);
            }
        }
    }

    async fn find_followers_inboxes(
        &mut self,
        user: &UserSpecifier,
    ) -> Result<Vec<InboxPair>, ServiceError<UserFindError>> {
        let user_id = self.find_user_by_specifier(user).await?.id;

        let follower_inboxes = sqlx::query_as!(
            InboxPair,
            r#"
            SELECT u.inbox, u.shared_inbox
            FROM user_follows uf
            INNER JOIN users u ON uf.follower_id = u.id
            WHERE uf.followee_id = ? AND (u.inbox IS NOT NULL OR u.shared_inbox IS NOT NULL)
            "#,
            user_id.to_string()
        )
        .fetch_all(&self.pool)
        .await?;

        Ok(follower_inboxes)
    }
}

pub struct DBSignerService {
    pool: MySqlPool,
    fetch: holder!(LocalUserFinderService),
    id_getter: IDGetterService,
}

impl DBSignerService {
    pub fn new(
        pool: MySqlPool,
        fetch: holder!(LocalUserFinderService),
        id_getter: IDGetterService,
    ) -> Self {
        Self {
            pool,
            fetch,
            id_getter,
        }
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

#[async_trait]
impl SignerService for DBSignerService {
    async fn fetch_signer(
        &mut self,
        user: &UserSpecifier,
    ) -> Result<holder!(ApubSigner), ServiceError<crate::services::SignerError>> {
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

        Ok(Box::new(DBSigner {
            user_id: u.id.to_string(),
            private_key: RsaPrivateKey::from_pkcs8_pem(private_key.as_ref())
                .map_err(|_| ServiceError::from_se(SignerError::PrivateKeyNotSet))?,
            private_key_id,
        }))
    }
}

#[derive(Constructor)]
pub struct DBUserProfileService {
    pool: MySqlPool,
    finder: holder!(LocalUserFinderService),
}

#[async_trait]
impl UserProfileService for DBUserProfileService {
    async fn update_user_profile(
        &mut self,
        spec: &UserSpecifier,
        update: &UserProfileUpdate,
    ) -> Result<(), ServiceError<anyhow::Error>> {
        let u = self
            .finder
            .find_user_by_specifier(spec)
            .await
            .map_err(|e| match e {
                ServiceError::SpecificError(LocalUserFindError::UserNotFound) => {
                    ServiceError::from_se(anyhow::anyhow!("user not found"))
                }
                _ => e.convert(),
            })?;

        sqlx::query!(
            r#"
            UPDATE users SET nickname=?, bio=?, avatar_id=? WHERE id=?
            "#,
            update.nickname,
            update.bio,
            update.avatar_id,
            u.id.to_string()
        )
        .execute(&self.pool)
        .await?;

        Ok(())
    }
}
