pub mod auth;
pub mod follow;
pub mod post;
pub mod user;

#[derive(Debug, Clone, PartialEq)]
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

#[derive(Debug, PartialEq, Eq, Clone)]
pub struct URI(String);

impl URI {
    pub fn from_str(uri: String) -> Option<Self> {
        Some(Self(uri))
    }
}
