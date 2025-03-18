use derive_more::{Constructor, From};
use reqwest::header::HeaderMap;
use serde::{Deserialize, Serialize};
use url::Url;

use super::{
    ServiceResult,
    id::{Identifier, NoteID, UserID},
};

pub mod note;

/// Fulltext search client
#[derive(Debug, Clone, Constructor)]
pub struct FTClient {
    client: reqwest_middleware::ClientWithMiddleware,
    api_key: String,
    content_locale: String,
    base_url: Url,
}

impl FTClient {
    pub fn client(&self) -> &reqwest_middleware::ClientWithMiddleware {
        &self.client
    }

    pub fn api_key(&self) -> &str {
        &self.api_key
    }

    pub fn content_locale(&self) -> &str {
        &self.content_locale
    }

    pub fn make_url(&self, path: impl AsRef<str>) -> Url {
        self.base_url.join(path.as_ref()).expect("invalid URL")
    }

    pub fn make_headers(&self) -> HeaderMap {
        let mut headers = HeaderMap::new();
        headers.insert("Content-Type", "application/json".parse().unwrap());
        headers.insert("X-TYPESENSE-API-KEY", self.api_key().parse().unwrap());
        headers
    }
}

#[derive(Debug, Clone, Copy)]
pub struct FulltextID<T: FullTextIdentifier>(T);

impl<T: FullTextIdentifier> FulltextID<T> {
    pub fn into_inner(self) -> T {
        self.0
    }

    pub fn inner(&self) -> &T {
        &self.0
    }
}

impl<T> Serialize for FulltextID<T>
where
    T: FullTextIdentifier,
{
    fn serialize<S>(&self, serializer: S) -> Result<S::Ok, S::Error>
    where
        S: serde::Serializer,
    {
        let kind = T::kind();
        let s = format!("{}-{}", kind, self.0);
        serializer.serialize_str(&s)
    }
}

impl<'de, T> Deserialize<'de> for FulltextID<T>
where
    T: FullTextIdentifier,
{
    fn deserialize<D>(deserializer: D) -> Result<Self, D::Error>
    where
        D: serde::Deserializer<'de>,
    {
        let s = String::deserialize(deserializer)?;

        // Split by the first hyphen
        let parts: Vec<&str> = s.splitn(2, '-').collect();
        if parts.len() != 2 {
            return Err(serde::de::Error::custom(
                "invalid format: expected KIND-VALUE",
            ));
        }

        let kind = parts[0];
        let value = parts[1];

        // Verify the kind matches
        if kind != T::kind() {
            return Err(serde::de::Error::custom(format!(
                "invalid kind: expected {}, got {}",
                T::kind(),
                kind
            )));
        }

        // Parse the value part into T
        match T::from_str(value) {
            Ok(t) => Ok(FulltextID(t)),
            Err(_) => Err(serde::de::Error::custom(format!(
                "failed to parse value: {}",
                value
            ))),
        }
    }
}

pub trait FullTextIdentifier: Identifier {
    fn kind() -> &'static str;
}

impl FullTextIdentifier for NoteID {
    fn kind() -> &'static str {
        "note"
    }
}

impl FullTextIdentifier for UserID {
    fn kind() -> &'static str {
        "user"
    }
}

#[derive(Debug, Clone, From)]
pub struct FTDateTime(pub chrono::DateTime<chrono::Utc>);

impl FTDateTime {
    pub fn into_inner(self) -> chrono::DateTime<chrono::Utc> {
        self.0
    }
}

impl Serialize for FTDateTime {
    fn serialize<S>(&self, serializer: S) -> Result<S::Ok, S::Error>
    where
        S: serde::Serializer,
    {
        let unix = self.0.timestamp();
        serializer.serialize_i64(unix)
    }
}

impl<'de> Deserialize<'de> for FTDateTime {
    fn deserialize<D>(deserializer: D) -> Result<Self, D::Error>
    where
        D: serde::Deserializer<'de>,
    {
        let unix = i64::deserialize(deserializer)?;
        let dt = chrono::DateTime::from_timestamp(unix, 0)
            .ok_or(serde::de::Error::custom("invalid timestamp"))?;
        Ok(FTDateTime(dt))
    }
}

pub async fn init(client: &FTClient) -> ServiceResult<()> {
    note::setup_note_collection(client).await?;
    Ok(())
}
