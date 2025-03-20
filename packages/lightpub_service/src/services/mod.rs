/*
Lightpub: a simple ActivityPub server
Copyright (C) 2025 tinaxd

This program is free software: you can redistribute it and/or modify
it under the terms of the GNU Affero General Public License as published
by the Free Software Foundation, either version 3 of the License, or
(at your option) any later version.

This program is distributed in the hope that it will be useful,
but WITHOUT ANY WARRANTY; without even the implied warranty of
MERCHANTABILITY or FITNESS FOR A PARTICULAR PURPOSE.  See the
GNU Affero General Public License for more details.

You should have received a copy of the GNU Affero General Public License
along with this program.  If not, see <https://www.gnu.org/licenses/>.
*/

use std::{borrow::Cow, error::Error, fmt::Debug};

use actix_web::{ResponseError, http::StatusCode};
use derive_more::From;
use serde_json::json;
use thiserror::Error;
use validator::ValidationErrors;

pub mod apub;
pub mod auth;
pub mod db;
pub mod follow;
pub mod fulltext;
pub mod id;
pub mod kv;
pub mod note;
pub mod notification;
pub mod queue;
pub mod search;
#[cfg(test)]
pub mod tests;
pub mod timeline;
pub mod upload;
pub mod user;

pub use expected_error::ExpectedError;

use crate::ServiceState;
pub type ServiceResult<T> = Result<T, ServiceError>;

macro_rules! internal_server_error_text {
    () => {
        "Internal Server Error"
    };
}
pub const INTERNAL_SERVER_ERROR_TEXT: &str = internal_server_error_text!();

pub async fn init_service(st: &ServiceState) -> ServiceResult<()> {
    if let Some(ft) = st.ft() {
        fulltext::init(ft).await?;
    }

    Ok(())
}

#[derive(Error, Debug)]
pub enum ServiceError {
    #[error("Known Error: {0}")]
    KnownError(Box<dyn ExpectedError + Send + Sync>),
    #[error("Unknown Error: {0}")]
    UnknownError(anyhow::Error),
    #[error("Validation error: {0}")]
    ValidationError(ValidationErrors),
    #[error("Internal Server Error: {0}")]
    InternalServerError(String),
}

impl ServiceError {
    pub fn known<E: ExpectedError + Send + Sync + 'static>(e: E) -> Self {
        Self::KnownError(Box::new(e))
    }

    pub fn unknown<E: Error + Send + Sync + 'static>(e: E) -> Self {
        Self::UnknownError(e.into())
    }

    pub fn unknown_box(e: Box<dyn Error + Send + Sync>) -> Self {
        Self::UnknownError(anyhow::Error::from_boxed(e))
    }

    pub fn validation(e: ValidationErrors) -> Self {
        Self::ValidationError(e)
    }

    pub fn ise(msg: impl Into<String>) -> Self {
        Self::InternalServerError(msg.into())
    }
}

#[macro_export]
macro_rules! impl_into_known {
    ($t:ty) => {
        impl From<$t> for ServiceError {
            fn from(e: $t) -> Self {
                ServiceError::known(e)
            }
        }
    };
}

impl ServiceError {
    pub fn get_error_code(&self) -> (StatusCode, Cow<str>) {
        match self {
            ServiceError::KnownError(e) => (e.status(), e.msg()),
            ServiceError::ValidationError(e) => (
                StatusCode::BAD_REQUEST,
                Cow::Owned(format!("Validation error: {:?}", e)),
            ),
            _ => (
                StatusCode::INTERNAL_SERVER_ERROR,
                Cow::Borrowed(INTERNAL_SERVER_ERROR_TEXT),
            ),
        }
    }
}

impl ResponseError for ServiceError {
    fn status_code(&self) -> StatusCode {
        self.get_error_code().0
    }

    fn error_response(&self) -> actix_web::HttpResponse<actix_web::body::BoxBody> {
        let (status, msg) = self.get_error_code();
        let body = json!({
            "error": msg
        });

        if status == StatusCode::INTERNAL_SERVER_ERROR {
            tracing::error!("Internal Server Error: {:?}", self);
        }

        actix_web::HttpResponse::build(status).json(body)
    }
}

#[derive(Debug, Error)]
#[error("{status}: {msg}")]
pub struct SimpleError {
    status: StatusCode,
    msg: Cow<'static, str>,
}

impl SimpleError {
    #[allow(dead_code)]
    fn new(status: StatusCode, msg: &'static str) -> Self {
        Self {
            status,
            msg: Cow::Borrowed(msg),
        }
    }

    fn new_owned(status: StatusCode, msg: String) -> Self {
        Self {
            status,
            msg: Cow::Owned(msg),
        }
    }
}

impl ExpectedError for SimpleError {
    fn status(&self) -> StatusCode {
        self.status
    }

    fn msg(&self) -> Cow<str> {
        self.msg.clone()
    }
}

pub fn create_error_simple<T>(status: StatusCode, msg: &'static str) -> ServiceResult<T> {
    Err(create_error_simple_err(status, msg))
}

pub fn create_error_simple_err(status: StatusCode, msg: impl Into<String>) -> ServiceError {
    ServiceError::known(SimpleError::new_owned(status, msg.into()))
}

#[derive(Debug, Error, From)]
pub enum FederationServiceError {
    #[error("{0}")]
    ServiceError(ServiceError),
    #[error("Federation Error: {0}")]
    FederationError(activitypub_federation::error::Error),
}

impl From<FederationServiceError> for ServiceError {
    fn from(value: FederationServiceError) -> Self {
        match value {
            FederationServiceError::ServiceError(e) => e,
            FederationServiceError::FederationError(fe) => fe.into(),
        }
    }
}

pub trait MapToUnknown
where
    Self: Sized,
{
    type Item;
    fn map_err_unknown(self) -> ServiceResult<Self::Item>;
}

impl<T, E> MapToUnknown for Result<T, E>
where
    E: Error + Send + Sync + 'static,
{
    type Item = T;
    fn map_err_unknown(self) -> ServiceResult<T> {
        self.map_err(|e| ServiceError::unknown(e))
    }
}

pub trait MapToKnown
where
    Self: Sized,
{
    type Item;
    fn map_err_known(self) -> ServiceResult<Self::Item>;
}

impl<T, E> MapToKnown for Result<T, E>
where
    E: Into<ServiceError> + Send + Sync + 'static,
{
    type Item = T;
    fn map_err_known(self) -> ServiceResult<T> {
        self.map_err(|e| e.into())
    }
}

impl From<activitypub_federation::error::Error> for ServiceError {
    fn from(value: activitypub_federation::error::Error) -> Self {
        use activitypub_federation::error::Error;
        match value {
            Error::NotFound => create_error_simple_err(StatusCode::NOT_FOUND, "Not Found"),
            Error::RequestLimit => {
                create_error_simple_err(StatusCode::TOO_MANY_REQUESTS, "Request Limit Exceeded")
            }
            Error::ObjectDeleted(id) => {
                create_error_simple_err(StatusCode::GONE, format!("Object {} is deleted", id))
            }
            Error::UrlVerificationError(e) => create_error_simple_err(StatusCode::BAD_REQUEST, e),
            Error::ActivityBodyDigestInvalid => {
                create_error_simple_err(StatusCode::BAD_REQUEST, "Activity Body Digest Invalid")
            }
            Error::ActivitySignatureInvalid => {
                create_error_simple_err(StatusCode::BAD_REQUEST, "Activity Signature Invalid")
            }
            Error::WebfingerResolveFailed(_e) => create_error_simple_err(
                StatusCode::BAD_REQUEST,
                "Failed to resolve actor via webfinger",
            ),
            Error::ParseFetchedObject(_e, url, _) => create_error_simple_err(
                StatusCode::BAD_REQUEST,
                format!("Failed to parse object fetched from {}", url),
            ),
            _ => ServiceError::unknown(value),
        }
    }
}

/// Upsert 時の値の設定方法を表現する。
#[derive(Debug, Clone, Copy)]
pub enum UpsertOperation<T> {
    /// Insert: Default 値を挿入する
    ///
    /// Update: 現在の値から変更しない
    KeepOrSetDefault,
    /// Insert: 指定された値で挿入する
    ///
    /// Update: 指定された値に変更する
    Set(T),
}
