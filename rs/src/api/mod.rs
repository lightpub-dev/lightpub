use std::fmt::Display;

use derive_getters::Getters;
use serde::{Deserialize, Serialize};
use serde_json::json;
use sqlx::SqlitePool;

use crate::application::service::{
    follow::FollowApplicationService,
    post::PostCreateApplicationService,
    user::{UserApplicationService, UserSecurityApplicationService},
};

pub mod model {
    use serde::Deserialize;

    macro_rules! Request {
        ($name:ident, $($field:ident $type:ty),*) => {
            #[derive(Debug, serde::Deserialize)]
            pub struct $name {
                $(
                    pub $field: $type,
                )*
            }
        }
    }

    macro_rules! Response {
        ($name:ident, $($field:ident $type:ty),*) => {
            #[derive(Debug, serde::Serialize)]
            pub struct $name {
                $(
                    pub $field: $type,
                )*
            }
        }
    }

    // User registration
    Request!(RegisterRequest,
        username String,
        nickname String,
        password String
    );
    Response!(RegisterResponse,
        user_id String
    );

    // User login
    Request!(LoginRequest,
        username String,
        password String
    );
    Response!(LoginResponse,
        token String
    );

    // Post create
    Request!(PostCreateRequest,
        content Option<String>,
        privacy String,
        reply_to_id Option<String>,
        repost_of_id Option<String>
    );

    Response!(PostCreateResponse,
        post_id String
    );

    // Webfinger
    #[derive(Debug, Deserialize)]
    pub struct WebfingerQuery {
        pub resource: String,
    }

    #[derive(Debug, Clone, PartialEq)]
    pub enum UserSpecifier {
        ID(String),
        Username(String, Option<String>),
    }

    impl std::fmt::Display for UserSpecifier {
        fn fmt(&self, f: &mut std::fmt::Formatter<'_>) -> std::fmt::Result {
            match self {
                UserSpecifier::ID(id) => write!(f, "{}", id),
                UserSpecifier::Username(username, hostname) => {
                    if let Some(hostname) = hostname {
                        write!(f, "@{}@{}", username, hostname)
                    } else {
                        write!(f, "@{}", username)
                    }
                }
            }
        }
    }

    #[derive(Debug, Clone)]
    pub enum UserSpecifierParseError {
        InvalidFormat,
    }

    impl UserSpecifier {
        pub fn parse(s: &str) -> Result<Self, UserSpecifierParseError> {
            if s.starts_with("@") {
                let parts: Vec<&str> = s[1..].split("@").collect();
                if parts.len() == 1 {
                    // no hostname
                    return Ok(UserSpecifier::Username(parts[0].to_string(), None));
                }
                if parts.len() == 2 {
                    return Ok(UserSpecifier::Username(
                        parts[0].to_string(),
                        Some(parts[1].to_string()),
                    ));
                }
                return Err(UserSpecifierParseError::InvalidFormat);
            }

            return Ok(UserSpecifier::ID(s.to_string()));
        }
    }

    impl<'de> Deserialize<'de> for UserSpecifier {
        fn deserialize<D>(deserializer: D) -> Result<Self, D::Error>
        where
            D: serde::Deserializer<'de>,
        {
            deserializer.deserialize_str(UserSpecifierVisitor {})
        }
    }

    struct UserSpecifierVisitor {}

    impl<'de> serde::de::Visitor<'de> for UserSpecifierVisitor {
        type Value = UserSpecifier;

        fn expecting(&self, formatter: &mut std::fmt::Formatter) -> std::fmt::Result {
            formatter.write_str("a user specifier")
        }

        fn visit_str<E>(self, value: &str) -> Result<UserSpecifier, E>
        where
            E: serde::de::Error,
        {
            UserSpecifier::parse(value).map_err(|e| E::custom(format!("{:?}", e)))
        }
    }
}

#[derive(Debug, Clone)]
pub struct AppState {
    config: AppConfig,
    pool: SqlitePool,
}

impl AppState {
    pub fn user_service(&self) -> UserApplicationService {
        // UserApplicationService::new()
        todo!()
    }

    pub fn user_security_service(&self) -> UserSecurityApplicationService {
        todo!()
    }

    pub fn post_create_service(&self) -> PostCreateApplicationService {
        todo!()
    }

    pub fn follow_service(&self) -> FollowApplicationService {
        todo!()
    }

    pub fn new(config: AppConfig, pool: SqlitePool) -> Self {
        Self { config, pool }
    }

    pub fn config(&self) -> &AppConfig {
        &self.config
    }

    pub fn pool(&self) -> &SqlitePool {
        &self.pool
    }
}

#[derive(Debug, Clone, Getters, Deserialize, Serialize)]
pub struct AppConfig {
    database: DatabaseConfig,
    base_url: String,
    dev: bool,
    upload_dir: String,
}

impl AppConfig {
    pub fn hostname(&self) -> &str {
        todo!()
    }
}

#[derive(Debug, Clone, Getters, Deserialize, Serialize)]
pub struct DatabaseConfig {
    url: String,
}

pub trait HasStatusCode: Display + std::fmt::Debug + Send + Sync {
    fn status_code(&self) -> i32;
    fn message(&self) -> String;
}

#[derive(Debug, Serialize)]
pub struct ErrorResponse {
    message: String,
    status: i32,
}

impl From<anyhow::Error> for ErrorResponse {
    fn from(value: anyhow::Error) -> Self {
        // check if value implements HasStatusCode
        if let Some(status_code) = value.downcast_ref::<Box<dyn HasStatusCode>>() {
            return ErrorResponse::new(status_code.status_code(), status_code.message());
        }

        // default is 500
        ErrorResponse::new(500, "internal server error".to_string())
    }
}

impl ErrorResponse {
    pub fn new(status: i32, message: impl Into<String>) -> Self {
        let msg = message.into();
        tracing::debug!("new error: {} {}", status, &msg);
        Self {
            message: msg.clone(),
            status,
        }
    }

    pub fn into_result<T>(self) -> Result<T, ErrorResponse> {
        Err(self)
    }
}

impl actix_web::error::ResponseError for ErrorResponse {
    fn status_code(&self) -> reqwest::StatusCode {
        reqwest::StatusCode::from_u16(self.status as u16).unwrap()
    }

    fn error_response(&self) -> actix_web::HttpResponse<actix_web::body::BoxBody> {
        actix_web::HttpResponse::build(self.status_code())
            .json(json!({
                "message": self.message
            }))
            .into()
    }
}

impl Display for ErrorResponse {
    fn fmt(&self, f: &mut std::fmt::Formatter<'_>) -> std::fmt::Result {
        write!(
            f,
            "{}",
            json!({ "message": self.message, "status": self.status })
        )
    }
}
