use sqlx::MySqlPool;
use thiserror::Error;
use uuid::Uuid;

use crate::{
    models,
    utils::{self, generate_uuid},
};

#[derive(Debug)]
pub struct DBUserService {
    pool: MySqlPool,
}

pub struct UserCreateRequest {
    username: String,
    email: String,
    password: String,
}

impl UserCreateRequest {
    pub fn new(username: String, email: String, password: String) -> Self {
        Self {
            username,
            email,
            password,
        }
    }
}

#[derive(Debug, Clone)]
pub struct UserLoginRequest {
    username: String,
    password: String,
}

#[derive(Debug, Clone)]
pub struct UserLoginResponse {
    token: String,
}

impl UserLoginRequest {
    pub fn new(username: String, password: String) -> Self {
        Self { username, password }
    }
}

impl UserLoginResponse {
    pub fn token(&self) -> &str {
        &self.token
    }
}

#[derive(Error, Debug)]
pub enum UserCreateError {
    #[error("username already taken")]
    UsernameTaken,
    #[error("DB error")]
    DBError(#[from] sqlx::Error),
}

#[derive(Error, Debug)]
pub enum UserLoginError {
    #[error("DB error")]
    DBError(#[from] sqlx::Error),
}

struct LoginDB {
    id: sqlx::types::Uuid,
    bpasswd: Option<String>,
}

impl DBUserService {
    pub fn new(pool: MySqlPool) -> Self {
        Self { pool }
    }

    pub async fn register_local_user(
        &mut self,
        req: &UserCreateRequest,
    ) -> Result<Uuid, UserCreateError> {
        let user_id = Uuid::new_v4();
        let user_id_str = utils::uuid_to_string(&user_id);

        // bcrypt
        let hashed = bcrypt::hash(req.password.clone(), bcrypt::DEFAULT_COST).unwrap();

        sqlx::query!(
            "INSERT INTO users (id, username, nickname, bpasswd) VALUES(?, ?, ?, ?)",
            user_id_str,
            req.username,
            req.email,
            hashed,
        )
        .execute(&self.pool)
        .await?;

        // TODO: conflict handlign

        Ok(user_id)
    }

    pub async fn login_user(
        &mut self,
        login: &UserLoginRequest,
    ) -> Result<Option<UserLoginResponse>, UserLoginError> {
        let user = sqlx::query_as!(
            LoginDB,
            "SELECT id, bpasswd FROM users WHERE username = ? AND host IS NULL",
            &login.username
        )
        .fetch_one(&self.pool)
        .await?;

        if let Some(bpasswd) = user.bpasswd {
            if bcrypt::verify(login.password.clone(), &bpasswd).unwrap() {
                let token = generate_uuid();
                sqlx::query!("INSERT INTO user_tokens (user_id, token) VALUES(?, ?)",)
            }
        }

        panic!()
    }
}
