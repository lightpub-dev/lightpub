use std::{
    collections::HashMap,
    sync::{Arc, Mutex},
};

use crate::{
    models::ApubSigner,
    services::apub::queue::transport::{encode_payload, PostToInboxPayload},
};
use async_trait::async_trait;
use futures::stream::StreamExt;
use lapin::{
    options::{
        BasicAckOptions, BasicConsumeOptions, BasicPublishOptions, ExchangeDeclareOptions,
        QueueDeclareOptions,
    },
    types::{FieldTable, ShortString},
    BasicProperties, ExchangeKind,
};

use crate::{
    holder,
    models::{
        apub::{Activity, Actor, CreatableObject},
        ApubWebfingerResponse,
    },
    services::{
        ApubFetchPostError, ApubFetchUserError, ApubRequestService, PostToInboxError, ServiceError,
        WebfingerError,
    },
};

use self::transport::{decode_payload, GetRequestPayload, GetWebfingerPayload, ResponsePayload};

pub mod transport {
    use std::fmt::Display;

    use rsa::RsaPrivateKey;
    use serde::{Deserialize, Serialize};
    use thiserror::Error;

    use crate::{
        models::apub::Activity,
        services::{MiscError, ServiceError},
    };

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
        Other(String),
    }

    #[derive(Debug, Error)]
    pub struct BackgroundProcessingError {
        pub message: String,
    }

    impl Display for BackgroundProcessingError {
        fn fmt(&self, f: &mut std::fmt::Formatter<'_>) -> std::fmt::Result {
            write!(f, "BackgroundProcessingError: {}", self.message)
        }
    }

    impl BackgroundProcessingError {
        pub fn new(message: &str) -> Self {
            Self {
                message: message.to_string(),
            }
        }
    }

    impl MiscError for BackgroundProcessingError {
        fn message(&self) -> &str {
            "internal server error"
        }

        fn status_code(&self) -> i32 {
            500
        }
    }

    impl<T, E> Into<Result<T, ServiceError<E>>> for ResponsePayload<T, E> {
        fn into(self) -> Result<T, ServiceError<E>> {
            match self {
                ResponsePayload::Success(t) => Ok(t),
                ResponsePayload::Failed(e) => Err(match e {
                    ResponseErrorPayload::Error(e) => ServiceError::SpecificError(e),
                    ResponseErrorPayload::Other(o) => {
                        ServiceError::MiscError(Box::new(BackgroundProcessingError::new(&o)))
                    }
                }),
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

#[derive(Debug)]
pub struct QueuedApubRequester {
    chan: lapin::Channel,

    response_tx_map: Arc<Mutex<HashMap<ShortString, tokio::sync::oneshot::Sender<Vec<u8>>>>>,
}

const POST_EXCHANGE: &str = "post";
const GET_REQUEST_EXCHANGE: &str = "get_request";
const RESPONSE_QUEUE: &str = "response";

const INBOX_POST_ROUTING_KEY: &str = "inbox.post";
const FETCH_USER_ROUTING_KEY: &str = "fetch.user";
const FETCH_POST_ROUTING_KEY: &str = "fetch.post";
const FETCH_WEBFINGER_ROUTING_KEY: &str = "fetch.webfinger";

const MAX_RETRY_HEADER: &str = "x-max-retry";
const RETRY_COUNT_HEADER: &str = "x-retry-count";
const INBOX_POST_MAX_RETRY: i32 = 10;

#[derive(Debug, Clone)]
pub struct QueuedApubRequesterBuilder {
    chan: lapin::Channel,
}

impl QueuedApubRequester {
    pub fn new(b: QueuedApubRequesterBuilder) -> Self {
        Self {
            chan: b.chan,
            response_tx_map: Arc::new(Mutex::new(HashMap::new())),
        }
    }

    pub async fn prepare(
        conn: lapin::Connection,
    ) -> Result<QueuedApubRequesterBuilder, lapin::Error> {
        let mut chan = conn.create_channel().await?;
        let response_tx_map = Arc::new(Mutex::new(HashMap::new()));
        Self::initialize(&mut chan, response_tx_map).await?;
        Ok(QueuedApubRequesterBuilder { chan })
    }

    async fn initialize(
        chan: &mut lapin::Channel,
        response_tx_map: Arc<Mutex<HashMap<ShortString, tokio::sync::oneshot::Sender<Vec<u8>>>>>,
    ) -> Result<(), lapin::Error> {
        let mut durable_options = ExchangeDeclareOptions::default();
        durable_options.durable = true;
        chan.exchange_declare(
            POST_EXCHANGE,
            ExchangeKind::Direct,
            durable_options,
            FieldTable::default(),
        )
        .await?;

        let default_options = ExchangeDeclareOptions::default();
        chan.exchange_declare(
            GET_REQUEST_EXCHANGE,
            ExchangeKind::Direct,
            default_options,
            FieldTable::default(),
        )
        .await?;

        chan.queue_declare(
            RESPONSE_QUEUE,
            QueueDeclareOptions::default(),
            FieldTable::default(),
        )
        .await?;

        let mut get_request_response_consumer = chan
            .basic_consume(
                RESPONSE_QUEUE,
                "",
                BasicConsumeOptions::default(),
                FieldTable::default(),
            )
            .await?;
        let response_tx_map = response_tx_map.clone();
        tokio::spawn(async move {
            while let Some(delivery) = get_request_response_consumer.next().await {
                let delivery = delivery.expect("error in consumer");
                delivery.ack(BasicAckOptions::default()).await.expect("ack");
                let id = delivery
                    .properties
                    .correlation_id()
                    .as_ref()
                    .expect("correlation_id not set");

                let mut map = response_tx_map.lock().unwrap();
                let tx = map.remove(id);
                if let Some(tx) = tx {
                    tx.send(delivery.data).expect("send response");
                }
            }
        });

        Ok(())
    }

    fn generate_random_id() -> String {
        let v4 = uuid::Uuid::new_v4();
        v4.simple().to_string()
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
        let chan = &mut self.chan;

        let mut headers = FieldTable::default();
        headers.insert(MAX_RETRY_HEADER.into(), INBOX_POST_MAX_RETRY.into());
        headers.insert(RETRY_COUNT_HEADER.into(), 0.into());

        let payload = PostToInboxPayload {
            url: url.to_string(),
            activity: activity.clone(),
            actor_id: actor.get_user_id(),
            actor_private_key: actor.get_private_key(),
            actor_key_id: actor.get_private_key_id(),
        };

        chan.basic_publish(
            POST_EXCHANGE,
            INBOX_POST_ROUTING_KEY,
            BasicPublishOptions::default(),
            &encode_payload(payload),
            BasicProperties::default().with_headers(headers),
        )
        .await?;

        Ok(())
    }

    async fn fetch_user(&mut self, url: &str) -> Result<Actor, ServiceError<ApubFetchUserError>> {
        let payload = GetRequestPayload {
            url: url.to_string(),
        };
        let response_id = Self::generate_random_id();

        self.chan
            .basic_publish(
                GET_REQUEST_EXCHANGE,
                FETCH_USER_ROUTING_KEY,
                BasicPublishOptions::default(),
                &encode_payload(payload),
                BasicProperties::default()
                    .with_reply_to(RESPONSE_QUEUE.into())
                    .with_correlation_id(response_id.clone().into()),
            )
            .await?;

        let (tx, rx) = tokio::sync::oneshot::channel();
        self.response_tx_map
            .lock()
            .unwrap()
            .insert(response_id.into(), tx);

        let response = rx.await.expect("response");
        let response: ResponsePayload<Actor, ApubFetchUserError> = decode_payload(&response);

        response.into()
    }

    async fn fetch_webfinger(
        &mut self,
        username: &str,
        host: &str,
    ) -> Result<ApubWebfingerResponse, ServiceError<WebfingerError>> {
        let payload = GetWebfingerPayload {
            username: username.to_string(),
            host: host.to_string(),
        };
        let response_id = Self::generate_random_id();

        self.chan
            .basic_publish(
                GET_REQUEST_EXCHANGE,
                FETCH_WEBFINGER_ROUTING_KEY,
                BasicPublishOptions::default(),
                &encode_payload(payload),
                BasicProperties::default()
                    .with_reply_to(RESPONSE_QUEUE.into())
                    .with_correlation_id(response_id.clone().into()),
            )
            .await?;

        let (tx, rx) = tokio::sync::oneshot::channel();
        self.response_tx_map
            .lock()
            .unwrap()
            .insert(response_id.into(), tx);

        let response = rx.await.expect("response");
        let response: ResponsePayload<ApubWebfingerResponse, WebfingerError> =
            decode_payload(&response);

        response.into()
    }

    async fn fetch_post(
        &mut self,
        url: &str,
    ) -> Result<CreatableObject, ServiceError<ApubFetchPostError>> {
        let payload = GetRequestPayload {
            url: url.to_string(),
        };
        let response_id = Self::generate_random_id();

        self.chan
            .basic_publish(
                GET_REQUEST_EXCHANGE,
                FETCH_POST_ROUTING_KEY,
                BasicPublishOptions::default(),
                &encode_payload(payload),
                BasicProperties::default()
                    .with_reply_to(RESPONSE_QUEUE.into())
                    .with_correlation_id(response_id.clone().into()),
            )
            .await?;

        let (tx, rx) = tokio::sync::oneshot::channel();
        self.response_tx_map
            .lock()
            .unwrap()
            .insert(response_id.into(), tx);

        let response = rx.await.expect("response");
        let response: ResponsePayload<CreatableObject, ApubFetchPostError> =
            decode_payload(&response);

        response.into()
    }
}
