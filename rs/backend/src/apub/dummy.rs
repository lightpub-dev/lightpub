use crate::ApubSigner;
use async_trait::async_trait;
use derive_more::Constructor;
use thiserror::Error;

use crate::{holder, Holder};
use crate::{
    ApubFetchPostError, ApubFetchUserError, ApubRequestService, MiscError, PostToInboxError,
    ServiceError, WebfingerError,
};
use lightpub_model::{
    apub::{Activity, Actor, CreatableObject},
    ApubWebfingerResponse,
};

#[derive(Constructor)]
pub struct DummyRequester {}

#[derive(Debug, Clone, Error)]
pub enum DummyError {
    #[error("dummy, no-op")]
    Dummy,
}

impl MiscError for DummyError {
    fn message(&self) -> &str {
        "internal server error"
    }

    fn status_code(&self) -> i32 {
        500
    }
}

#[async_trait]
impl ApubRequestService for DummyRequester {
    async fn post_to_inbox(
        &mut self,
        _url: &str,
        _activity: &Activity,
        _actor: holder!(ApubSigner),
    ) -> Result<(), ServiceError<PostToInboxError>> {
        Ok(())
    }

    async fn fetch_user(&mut self, _url: &str) -> Result<Actor, ServiceError<ApubFetchUserError>> {
        Err(ServiceError::MiscError(Box::new(DummyError::Dummy)))
    }

    async fn fetch_webfinger(
        &mut self,
        _username: &str,
        _host: &str,
    ) -> Result<ApubWebfingerResponse, ServiceError<WebfingerError>> {
        Err(ServiceError::MiscError(Box::new(DummyError::Dummy)))
    }

    async fn fetch_post(
        &mut self,
        _url: &str,
    ) -> Result<CreatableObject, ServiceError<ApubFetchPostError>> {
        Err(ServiceError::MiscError(Box::new(DummyError::Dummy)))
    }
}
