use std::fmt::Display;

pub mod auth;
pub mod ctx;
pub mod follow;
pub mod post;
pub mod user;

#[derive(Debug, Clone, PartialEq, sqlx::Type)]
#[sqlx(transparent)]
pub struct DateTime(chrono::DateTime<chrono::Utc>);

impl DateTime {
    pub fn from_utc(dt: chrono::DateTime<chrono::Utc>) -> Self {
        Self(dt)
    }

    pub fn from_naive_as_utc(naive: chrono::NaiveDateTime) -> Self {
        Self(chrono::DateTime::<chrono::Utc>::from_naive_utc_and_offset(
            naive,
            chrono::Utc,
        ))
    }

    pub fn to_utc(&self) -> chrono::DateTime<chrono::Utc> {
        self.0
    }

    pub fn now() -> Self {
        Self(chrono::Utc::now())
    }
}

#[derive(Debug, PartialEq, Eq, Clone, sqlx::Type)]
pub struct URI(String);

impl URI {
    pub fn from_str(uri: impl Into<String>) -> Option<Self> {
        Some(Self(uri.into()))
    }

    pub fn to_str(&self) -> &str {
        &self.0
    }
}

impl Display for URI {
    fn fmt(&self, f: &mut std::fmt::Formatter<'_>) -> std::fmt::Result {
        write!(f, "{}", self.0)
    }
}
