use std::fmt::Debug;

use crate::backend::{
    ApubFetchPostError, ApubFetchUserError, ApubRequestService, MiscError, PostToInboxError,
    ServiceError, WebfingerError,
};
use crate::holder;
use crate::model::{
    apub::{Activity, Actor, CreatableObject},
    ApubSigner, ApubWebfingerResponse,
};

use self::transport::{decode_payload, ResponsePayload};
use crate::model::queue::{PostToInboxPayload as PostToInboxQueuePayload, SignerPayload};
use async_trait::async_trait;
use futures::Future;
use serde::Deserialize;

pub mod transport {
    use std::fmt::Display;

    use rsa::RsaPrivateKey;
    use serde::{Deserialize, Serialize};
    use thiserror::Error;

    use crate::backend::{MiscError, ServiceError};
    use crate::model::apub::Activity;

    #[derive(Debug, Serialize, Deserialize)]
    pub struct PostToInboxPayload {
        pub url: String,
        pub activity: Activity,
        pub actor_id: String,
        pub actor_private_key: RsaPrivateKey,
        pub actor_key_id: String,
    }

    #[derive(Debug, Serialize, Deserialize)]
    pub struct GetRequestPayload {
        pub url: String,
    }

    #[derive(Debug, Serialize, Deserialize)]
    pub struct GetWebfingerPayload {
        pub username: String,
        pub host: String,
    }

    #[derive(Debug, Serialize, Deserialize)]
    pub enum ResponsePayload<T, E> {
        Success(T),
        Failed(ResponseErrorPayload<E>),
    }

    #[derive(Debug, Serialize, Deserialize, Error)]
    pub enum ResponseErrorPayload<E> {
        #[error("Error: {0}")]
        Error(E),
        #[error("MiscError: {0}")]
        Other(String, i32),
    }

    #[derive(Debug, Error)]
    pub struct BackgroundProcessingError {
        pub message: String,
        pub status: i32,
    }

    impl Display for BackgroundProcessingError {
        fn fmt(&self, f: &mut std::fmt::Formatter<'_>) -> std::fmt::Result {
            write!(f, "BackgroundProcessingError: {}", self.message)
        }
    }

    impl BackgroundProcessingError {
        pub fn new(message: &str, status: i32) -> Self {
            Self {
                message: message.to_string(),
                status: status,
            }
        }
    }

    impl MiscError for BackgroundProcessingError {
        fn message(&self) -> &str {
            "internal server error"
        }

        fn status_code(&self) -> i32 {
            self.status
        }
    }

    impl<T, E> Into<Result<T, ServiceError<E>>> for ResponsePayload<T, E> {
        fn into(self) -> Result<T, ServiceError<E>> {
            match self {
                ResponsePayload::Success(t) => Ok(t),
                ResponsePayload::Failed(e) => Err(match e {
                    ResponseErrorPayload::Error(e) => ServiceError::SpecificError(e),
                    ResponseErrorPayload::Other(o, s) => {
                        ServiceError::MiscError(Box::new(BackgroundProcessingError::new(&o, s)))
                    }
                }),
            }
        }
    }

    impl<T, E> From<Result<T, ServiceError<E>>> for ResponsePayload<T, E> {
        fn from(value: Result<T, ServiceError<E>>) -> Self {
            match value {
                Ok(v) => ResponsePayload::Success(v),
                Err(e) => match e {
                    ServiceError::SpecificError(e) => {
                        ResponsePayload::Failed(ResponseErrorPayload::Error(e))
                    }
                    ServiceError::MiscError(e) => ResponsePayload::Failed(
                        ResponseErrorPayload::Other(e.message().to_string(), e.status_code()),
                    ),
                },
            }
        }
    }

    pub fn encode_payload<T: Serialize>(payload: T) -> Vec<u8> {
        serde_json::to_vec(&payload).unwrap()
    }

    pub fn decode_payload<T: for<'de> Deserialize<'de>>(payload: &[u8]) -> T {
        serde_json::from_slice(payload).unwrap()
    }
}

pub mod worker {}

pub struct QueuedApubRequester {
    pool: sqlx::Pool<sqlx::Sqlite>,
    requester: holder!(ApubRequestService),
}

#[derive(Debug, Clone)]
pub struct QueuedApubRequesterBuilder {}

async fn report_warn<R, E: Debug, T: Future<Output = Result<R, E>>>(task: T) {
    let result = task.await;
    match result {
        Ok(_) => {}
        Err(e) => {
            tracing::warn!("error: {:?}", e);
        }
    }
}

impl QueuedApubRequester {
    pub fn new(pool: sqlx::Pool<sqlx::Sqlite>, requester: holder!(ApubRequestService)) -> Self {
        Self { pool, requester }
    }

    fn generate_random_id() -> String {
        let v4 = uuid::Uuid::new_v4();
        v4.simple().to_string()
    }
}

#[derive(Debug)]
pub struct TaskTimeoutError {}

impl MiscError for TaskTimeoutError {
    fn message(&self) -> &str {
        "server is too busy"
    }

    fn status_code(&self) -> i32 {
        503 // Service Unavailable
    }
}

impl TaskTimeoutError {
    pub fn new() -> Self {
        Self {}
    }
}

async fn wait_for_result<T, E>(
    rx: tokio::sync::oneshot::Receiver<Vec<u8>>,
    timeout: std::time::Duration,
) -> Result<T, ServiceError<E>>
where
    T: for<'de> Deserialize<'de> + Debug,
    E: for<'de> Deserialize<'de> + Debug,
{
    let result = tokio::time::timeout(timeout, rx).await;

    match result {
        Ok(r) => match r {
            Ok(r) => {
                let decoded: ResponsePayload<T, E> = decode_payload(&r);
                tracing::debug!("got response from worker: {:?}", &decoded);
                decoded.into()
            }
            Err(e) => {
                tracing::error!("failed to receive response from worker: {:?}", e);
                Err(ServiceError::MiscError(Box::new(TaskTimeoutError::new())))
            }
        },
        Err(e) => {
            tracing::warn!("worker task timeout: {:?}", e);
            Err(ServiceError::MiscError(Box::new(TaskTimeoutError::new())))
        }
    }
}

#[async_trait]
impl ApubRequestService for QueuedApubRequester {
    async fn post_to_inbox(
        &mut self,
        url: &str,
        activity: &Activity,
        actor: holder!(ApubSigner),
    ) -> Result<(), ServiceError<PostToInboxError>> {
        let payload = PostToInboxQueuePayload {
            url: url.to_string(),
            activity: activity.clone(),
            actor: SignerPayload {
                user_id: actor.get_user_id(),
                private_key: actor.get_private_key(),
                private_key_id: actor.get_private_key_id(),
            },
        };

        let payload_text = serde_json::ser::to_string(&payload).unwrap();
        sqlx::query!("INSERT INTO QueuedTask(payload) VALUES (?)", payload_text)
            .execute(&self.pool)
            .await
            .unwrap();

        Ok(())
    }

    async fn fetch_user(&mut self, url: &str) -> Result<Actor, ServiceError<ApubFetchUserError>> {
        self.requester.fetch_user(url).await
    }

    async fn fetch_webfinger(
        &mut self,
        username: &str,
        host: &str,
    ) -> Result<ApubWebfingerResponse, ServiceError<WebfingerError>> {
        self.requester.fetch_webfinger(username, host).await
    }

    async fn fetch_post(
        &mut self,
        url: &str,
    ) -> Result<CreatableObject, ServiceError<ApubFetchPostError>> {
        self.requester.fetch_post(url).await
    }
}
