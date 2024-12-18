use crate::backend::UserLogoutError;
use crate::model::User;
use async_trait::async_trait;
use derive_more::Constructor;
use gen_span::gen_span;
use opentelemetry::global;
use opentelemetry::trace::Span;
use opentelemetry::trace::Tracer;
use rsa::pkcs8::DecodePrivateKey;
use rsa::RsaPrivateKey;
use sqlx::SqlitePool;
use tracing::error;
use tracing::warn;
use uuid::fmt::Simple;
use uuid::Uuid;

use crate::backend::id::IDGetterService;
use crate::backend::id::UserAttribute;
use crate::backend::AllUserFinderService;
use crate::backend::ApubRequestService;
use crate::backend::AuthError;
use crate::backend::InboxPair;
use crate::backend::LocalUserFindError;
use crate::backend::LocalUserFinderService;
use crate::backend::ServiceError;
use crate::backend::SignerError;
use crate::backend::SignerService;
use crate::backend::UserAuthService;
use crate::backend::UserCreateError;
use crate::backend::UserCreateRequest;
use crate::backend::UserCreateResult;
use crate::backend::UserCreateService;
use crate::backend::UserFindError;
use crate::backend::UserLoginError;
use crate::backend::UserLoginRequest;
use crate::backend::UserLoginResult;
use crate::backend::UserProfileService;
use crate::backend::UserProfileUpdate;
use crate::holder;
use crate::model::apub::Actor;
use crate::model::ApubSigner;
use crate::model::UserSpecifier;
use crate::utils::generate_uuid;
use crate::utils::generate_uuid_random;
use rsa::pkcs8::{EncodePrivateKey, EncodePublicKey};
use std::str::FromStr;

#[derive(Debug)]
pub struct DBUserCreateService {
    pool: SqlitePool,
}

impl DBUserCreateService {
    pub fn new(pool: SqlitePool) -> Self {
        Self { pool }
    }
}

#[derive(Debug)]
pub struct DBAuthService {
    pool: SqlitePool,
}

impl DBAuthService {
    pub fn new(pool: SqlitePool) -> Self {
        Self { pool }
    }
}

struct LoginDB {
    id: Simple,
    bpasswd: Option<String>,
}

#[gen_span]
#[async_trait]
impl UserCreateService for DBUserCreateService {
    async fn create_user(
        &mut self,
        req: &UserCreateRequest,
    ) -> Result<UserCreateResult, ServiceError<UserCreateError>> {
        let trace = global::tracer("");

        let mut span = trace.start("existence_check");
        let exist = sqlx::query!(
            r#"
        SELECT COUNT(*) AS count FROM users WHERE username=? AND host IS NULL
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
        span.end();

        let user_id = Uuid::new_v4().simple();

        // bcrypt
        let hashed = trace.in_span("password_hashing", |_| {
            bcrypt::hash(req.password.clone(), bcrypt::DEFAULT_COST).unwrap()
        });

        // generate rsa keys
        let mut span = trace.start("rsa_keygen");
        let (private_key, public_key) = tokio::task::spawn_blocking(|| {
            let private_key = crate::utils::key::generate();
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
        span.end();

        let mut span = trace.start("db_insert");
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
        span.end();

        // TODO: conflict handlign

        Ok(UserCreateResult { user_id })
    }

    async fn login_user(
        &mut self,
        req: &UserLoginRequest,
    ) -> Result<UserLoginResult, ServiceError<UserLoginError>> {
        let tracer = global::tracer("");

        let mut span = tracer.start("db_select");
        let user = sqlx::query_as!(
            LoginDB,
            "SELECT id AS `id!: Simple`, bpasswd FROM users WHERE username = ? AND host IS NULL",
            req.username
        )
        .fetch_optional(&self.pool)
        .await?;

        let user = if let Some(u) = user {
            u
        } else {
            return Err(ServiceError::from_se(UserLoginError::AuthFailed));
        };
        span.end();

        if let Some(bpasswd) = user.bpasswd {
            let mut span = tracer.start("bcrypt_verify");
            if bcrypt::verify(req.password.clone(), &bpasswd).unwrap() {
                span.end();
                let mut span = tracer.start("token_gen");
                let token = generate_uuid_random();
                sqlx::query!(
                    "INSERT INTO user_tokens (user_id, token) VALUES(?, ?)",
                    user.id,
                    token
                )
                .execute(&self.pool)
                .await?;
                span.end();
                return Ok(UserLoginResult { user_token: token });
            }
            return Err(ServiceError::SpecificError(UserLoginError::AuthFailed));
        } else {
            return Err(ServiceError::SpecificError(UserLoginError::AuthFailed));
        }
    }

    async fn logout_user(&mut self, token: &str) -> Result<(), ServiceError<UserLogoutError>> {
        sqlx::query!("DELETE FROM user_tokens WHERE token=?", token)
            .execute(&self.pool)
            .await?;
        Ok(())
    }
}

#[gen_span]
#[async_trait]
impl UserAuthService for DBAuthService {
    async fn authenticate_user(
        &mut self,
        token: &str,
    ) -> Result<User, ServiceError<crate::backend::AuthError>> {
        let u = sqlx::query_as!(User,
            "SELECT users.id AS `id!: Simple`, username, host, nickname, bio, uri, shared_inbox, inbox, outbox, public_key, users.created_at AS `created_at: chrono::NaiveDateTime` FROM users INNER JOIN user_tokens ON users.id = user_tokens.user_id WHERE token = ?",
            token
        ).fetch_one(&self.pool).await;

        match u {
            Ok(u) => {
                return Ok(u);
            }
            Err(e) => match e {
                sqlx::Error::RowNotFound => {
                    return Err(ServiceError::from_se(AuthError::TokenNotSet));
                }
                _ => {
                    error!("Error during authentication: {:?}", e);
                    return Err(ServiceError::from_se(AuthError::Other));
                }
            },
        }
    }
}

#[derive(Debug)]
pub struct DBLocalUserFinderService {
    pool: SqlitePool,
}

impl DBLocalUserFinderService {
    pub fn new(pool: SqlitePool) -> Self {
        Self { pool }
    }
}

#[gen_span]
#[async_trait]
impl LocalUserFinderService for DBLocalUserFinderService {
    async fn find_user_by_specifier(
        &mut self,
        spec: &UserSpecifier,
    ) -> Result<User, ServiceError<LocalUserFindError>> {
        match spec {
            UserSpecifier::Username(username, host) => {
                if let Some(_) = host {
                    return Err(ServiceError::from_se(
                        LocalUserFindError::NotLocalUser.into(),
                    ));
                }

                let u = sqlx::query_as!(User,
                "SELECT id AS `id!: Simple`, username, host, nickname, bio, uri, shared_inbox, inbox, outbox, public_key, created_at AS `created_at: chrono::NaiveDateTime` FROM users WHERE username = ? AND host IS NULL", username).fetch_one(&self.pool).await?;
                return Ok(u);
            }
            UserSpecifier::URL(_url) => {
                return Err(ServiceError::from_se(LocalUserFindError::NotLocalUser));
            }
            UserSpecifier::ID(id) => {
                let id_str = id.simple().to_string();
                let u = sqlx::query_as!(User,
                "SELECT id AS `id!: Simple`, username, host, nickname, bio, uri, shared_inbox, inbox, outbox, public_key, created_at AS `created_at: chrono::NaiveDateTime` FROM users WHERE id = ?", id_str).fetch_one(&self.pool).await?;
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
    pool: SqlitePool,
    req: holder!(ApubRequestService),
    local: holder!(LocalUserFinderService),
    id_getter: IDGetterService,
}

impl DBAllUserFinderService {
    pub fn new(
        pool: SqlitePool,
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

fn process_public_key_pem(pem: &str) -> String {
    // Pleroma sends public key with a extra newline at the end,
    // which leads to public key import failure.
    // Trim all newlines from the end,
    // then append a newline
    let trimmed = pem.trim_end_matches('\n');
    format!("{}\n", trimmed)
}

#[gen_span]
async fn find_user_by_url(
    url: impl Into<String>,
    req: &mut holder!(ApubRequestService),
    pool: &SqlitePool,
) -> Result<User, ServiceError<crate::backend::UserFindError>> {
    let url = url.into();
    // we can assume that the url is a remote-user url

    // try to fetch user from db
    let u = sqlx::query_as!(User,
        "SELECT id AS `id!: Simple`, username, host, nickname, bio, uri, shared_inbox, inbox, outbox, public_key, created_at AS `created_at: chrono::NaiveDateTime` FROM users WHERE uri = ?", url).fetch_optional(pool).await?;
    if let Some(u) = u {
        // try to fetch remote_users
        let user_id_str = u.id.to_string();
        let ru = sqlx::query_as!(RemoteUser,
            "SELECT user_id AS `user_id!: Simple`, following, followers, liked, fetched_at AS `fetched_at: chrono::NaiveDateTime` FROM remote_users WHERE user_id = ?", user_id_str).fetch_optional(pool).await?;
        if let Some(ru) = ru {
            // check if info is up-to-date
            let now = chrono::Utc::now().naive_utc();
            if (now - ru.fetched_at) < chrono::Duration::try_hours(1).unwrap() {
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
    let pubkey_pem = process_public_key_pem(&pubkey.public_key_pem);
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

    let user = User {
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
    let user_id_str = user.id.to_string();
    sqlx::query!(
        "INSERT INTO users (id, username, host, bpasswd, nickname, uri, shared_inbox, inbox, outbox, bio) VALUES (?,?,?,NULL,?,?,?,?,?,?) ON CONFLICT DO UPDATE SET username = ?, host = ?, bpasswd = NULL, nickname = ?, uri = ?, shared_inbox = ?, inbox = ?, outbox = ?, bio = ?",
        user_id_str,
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

    let user_id_str = ru.user_id.to_string();
    sqlx::query!(
        "INSERT INTO remote_users(user_id, following, followers, liked, fetched_at) VALUES(?,?,?,?,?) ON CONFLICT DO UPDATE SET following = ?, followers = ?, liked = ?, fetched_at = ?",
        user_id_str,
        ru.following,
        ru.followers,
        ru.liked,
        ru.fetched_at,
        ru.following,
        ru.followers,
        ru.liked,
        ru.fetched_at,
    ).execute(&mut *tx).await?;

    let user_id_str = user_id.to_string();
    sqlx::query!("
        INSERT INTO user_keys (id, owner_id, public_key) VALUES (?,?,?) ON CONFLICT DO UPDATE SET owner_id=?, public_key=?, updated_at=DATETIME('now')
    ", pubkey_id, user_id_str, pubkey_pem, user_id_str, pubkey_pem).execute(&mut *tx).await?;

    tx.commit().await?;

    return Ok(user);
}

#[gen_span]
#[async_trait]
impl AllUserFinderService for DBAllUserFinderService {
    async fn find_user_by_specifier(
        &mut self,
        spec: &UserSpecifier,
    ) -> Result<User, ServiceError<crate::backend::UserFindError>> {
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
                    let u = sqlx::query_as!(User,
                        "SELECT id AS `id!: Simple`, username, host, nickname, bio, uri, shared_inbox, inbox, outbox, public_key, created_at AS `created_at: chrono::NaiveDateTime` FROM users WHERE username = ? AND host = ?", username, host).fetch_optional(&self.pool).await?;
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
                let id_str = id.simple().to_string();
                let u = sqlx::query_as!(User,
                    "SELECT id AS `id!: Simple`, username, host, nickname, bio, uri, shared_inbox, inbox, outbox, public_key, created_at AS `created_at: chrono::NaiveDateTime` FROM users WHERE id = ?", id_str).fetch_one(&self.pool).await?;

                return Ok(u);
            }
        }
    }

    async fn find_followers_inboxes(
        &mut self,
        user: &UserSpecifier,
    ) -> Result<Vec<InboxPair>, ServiceError<UserFindError>> {
        let user_id = self.find_user_by_specifier(user).await?.id;

        let user_id_str = user_id.to_string();
        let follower_inboxes = sqlx::query_as!(
            InboxPair,
            r#"
            SELECT u.inbox, u.shared_inbox
            FROM user_follows uf
            INNER JOIN users u ON uf.follower_id = u.id
            WHERE uf.followee_id = ? AND (u.inbox IS NOT NULL OR u.shared_inbox IS NOT NULL)
            "#,
            user_id_str
        )
        .fetch_all(&self.pool)
        .await?;

        Ok(follower_inboxes)
    }
}

pub struct DBSignerService {
    pool: SqlitePool,
    fetch: holder!(LocalUserFinderService),
    id_getter: IDGetterService,
}

impl DBSignerService {
    pub fn new(
        pool: SqlitePool,
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

#[gen_span]
#[async_trait]
impl SignerService for DBSignerService {
    async fn fetch_signer(
        &mut self,
        user: &UserSpecifier,
    ) -> Result<holder!(ApubSigner), ServiceError<crate::backend::SignerError>> {
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

        let u_id_str = u.id.to_string();
        let private_key = sqlx::query!("SELECT private_key FROM users WHERE id = ?", u_id_str)
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
    pool: SqlitePool,
    finder: holder!(LocalUserFinderService),
}

#[gen_span]
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

        let u_id_str = u.id.to_string();
        sqlx::query!(
            r#"
            UPDATE users SET nickname=?, bio=?, avatar_id=? WHERE id=?
            "#,
            update.nickname,
            update.bio,
            update.avatar_id,
            u_id_str
        )
        .execute(&self.pool)
        .await?;

        Ok(())
    }
}
