use std::{
    collections::HashMap,
    fmt::Debug,
    sync::{Arc, Mutex},
};

use crate::{
    apub::queue::transport::{encode_payload, PostToInboxPayload},
    holder, ApubFetchPostError, ApubFetchUserError, ApubRequestService, MiscError,
    PostToInboxError, ServiceError, WebfingerError,
};
use lightpub_model::{
    apub::{Activity, Actor, CreatableObject},
    ApubSigner, ApubWebfingerResponse,
};

use async_trait::async_trait;
use futures::{stream::StreamExt, Future};
use lapin::{
    options::{
        BasicAckOptions, BasicConsumeOptions, BasicPublishOptions, BasicRejectOptions,
        ExchangeBindOptions, ExchangeDeclareOptions, QueueBindOptions, QueueDeclareOptions,
    },
    types::{FieldTable, LongString},
    BasicProperties, ExchangeKind,
};
use serde::Deserialize;

use self::transport::{decode_payload, GetRequestPayload, GetWebfingerPayload, ResponsePayload};

pub mod transport {
    use std::fmt::Display;

    use rsa::RsaPrivateKey;
    use serde::{Deserialize, Serialize};
    use thiserror::Error;

    use crate::{MiscError, ServiceError};
    use lightpub_model::apub::Activity;

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

pub mod worker {
    use futures::StreamExt;
    use lapin::{
        options::{
            BasicAckOptions, BasicPublishOptions, BasicRejectOptions, QueueBindOptions,
            QueueDeclareOptions,
        },
        types::{FieldTable, LongString},
        BasicProperties,
    };
    use reqwest::{Method, Request, RequestBuilder};
    use rsa::RsaPrivateKey;
    use tracing::info;

    use crate::{
        apub::{map_error, ApubReqwestError, ApubReqwester, WebfingerResponse},
        holder, ApubFetchPostError, ApubFetchUserError, ServiceError, WebfingerError,
    };
    use lightpub_model::{
        apub::{context::ContextAttachable, Activity, Actor, CreatableObject},
        ApubSigner, ApubWebfingerResponse, ApubWebfingerResponseBuilder,
    };
    use lightpub_utils::key::{attach_signature, SignKeyBuilder};

    use super::{
        transport::{
            decode_payload, encode_payload, GetRequestPayload, GetWebfingerPayload,
            PostToInboxPayload, ResponsePayload,
        },
        DLX_HEADER, FETCH_POST_ROUTING_KEY, FETCH_USER_ROUTING_KEY, FETCH_WEBFINGER_ROUTING_KEY,
        GET_REQUEST_EXCHANGE, INBOX_POST_ROUTING_KEY, POST_DLX, POST_EXCHANGE,
    };

    const GET_QUEUE: &str = "processing_get"; // TODO: queues should be separated for more important messages to be processed more quickly
    const POST_QUEUE: &str = "processing_post";

    pub struct ApubWorker {
        chan: lapin::Channel,
        client: ApubReqwester,
    }

    #[derive(Debug)]
    struct SimpleSigner {
        user_id: String,
        key_id: String,
        private_key: RsaPrivateKey,
    }

    impl ApubSigner for SimpleSigner {
        fn get_private_key(&self) -> RsaPrivateKey {
            self.private_key.clone()
        }

        fn get_private_key_id(&self) -> String {
            self.key_id.clone()
        }

        fn get_user_id(&self) -> String {
            self.user_id.clone()
        }
    }

    impl ApubWorker {
        pub fn new(chan: lapin::Channel, client: ApubReqwester) -> Self {
            Self { chan, client }
        }

        pub async fn start(mut self, queue: &str) {
            let mut consumer = self
                .chan
                .basic_consume(queue, "", Default::default(), Default::default())
                .await
                .unwrap();
            tracing::info!("[WORKER] background worker started");
            while let Some(msg) = consumer.next().await {
                match msg {
                    Err(e) => {
                        tracing::error!("failed to receive message: {:?}", e);
                        continue;
                    }
                    Ok(msg) => match msg.routing_key.as_str() {
                        INBOX_POST_ROUTING_KEY => {
                            tracing::debug!("[WORKER] {} received", msg.routing_key.as_str());
                            let payload: PostToInboxPayload = decode_payload(&msg.data);
                            let result = self
                                .post_to_inbox(
                                    &payload.url,
                                    &payload.activity,
                                    Box::new(SimpleSigner {
                                        user_id: payload.actor_id,
                                        key_id: payload.actor_key_id,
                                        private_key: payload.actor_private_key,
                                    }),
                                )
                                .await;
                            match result {
                                Ok(_) => msg.ack(BasicAckOptions::default()).await.unwrap(),
                                Err(e) => {
                                    tracing::warn!("post_to_inbox error: {:?}", e);
                                    let mut options = BasicRejectOptions::default();
                                    options.requeue = false;
                                    msg.reject(options).await.unwrap()
                                }
                            }
                        }
                        FETCH_USER_ROUTING_KEY => {
                            tracing::debug!("[WORKER] {} received", msg.routing_key.as_str());
                            let payload: GetRequestPayload = decode_payload(&msg.data);
                            let result = self.fetch_user(&payload.url).await;
                            let response: ResponsePayload<Actor, ApubFetchUserError> =
                                result.into();
                            self.chan
                                .basic_publish(
                                    "",
                                    msg.properties.reply_to().as_ref().unwrap().as_str(),
                                    BasicPublishOptions::default(),
                                    &encode_payload(response),
                                    BasicProperties::default().with_correlation_id(
                                        msg.properties.correlation_id().as_ref().unwrap().clone(),
                                    ),
                                )
                                .await
                                .unwrap();
                            msg.ack(BasicAckOptions::default()).await.unwrap();
                        }
                        FETCH_POST_ROUTING_KEY => {
                            tracing::debug!("[WORKER] {} received", msg.routing_key.as_str());
                            let payload: GetRequestPayload = decode_payload(&msg.data);
                            let result = self.fetch_post(&payload.url).await;
                            let response: ResponsePayload<CreatableObject, ApubFetchPostError> =
                                result.into();
                            self.chan
                                .basic_publish(
                                    "",
                                    msg.properties.reply_to().as_ref().unwrap().as_str(),
                                    BasicPublishOptions::default(),
                                    &encode_payload(response),
                                    BasicProperties::default().with_correlation_id(
                                        msg.properties.correlation_id().as_ref().unwrap().clone(),
                                    ),
                                )
                                .await
                                .unwrap();
                            msg.ack(BasicAckOptions::default()).await.unwrap();
                        }
                        FETCH_WEBFINGER_ROUTING_KEY => {
                            tracing::debug!("[WORKER] {} received", msg.routing_key.as_str());
                            let payload: GetWebfingerPayload = decode_payload(&msg.data);
                            let result =
                                self.fetch_webfinger(&payload.username, &payload.host).await;
                            let response: ResponsePayload<ApubWebfingerResponse, WebfingerError> =
                                result.into();
                            self.chan
                                .basic_publish(
                                    "",
                                    msg.properties.reply_to().as_ref().unwrap().as_str(),
                                    BasicPublishOptions::default(),
                                    &encode_payload(response),
                                    BasicProperties::default().with_correlation_id(
                                        msg.properties.correlation_id().as_ref().unwrap().clone(),
                                    ),
                                )
                                .await
                                .unwrap();
                            msg.ack(BasicAckOptions::default()).await.unwrap();
                        }
                        s => {
                            tracing::error!("unknown routing key! {:?}", s);
                            continue;
                        }
                    },
                }
            }
        }

        fn client(&self) -> reqwest::Client {
            self.client.client.clone()
        }

        async fn post_to_inbox(
            &mut self,
            url: &str,
            activity: &Activity,
            actor: holder!(ApubSigner),
        ) -> Result<(), super::ServiceError<super::PostToInboxError>> {
            let body = serde_json::to_string(&activity.with_context()).unwrap();
            info!("body: {:?}", &body);

            let client = self.client();

            let mut req = RequestBuilder::from_parts(
                self.client(),
                Request::new(Method::POST, url.parse().unwrap()),
            )
            .header(
                "Content-Type",
                r#"application/ld+json; profile="https://www.w3.org/ns/activitystreams""#,
            )
            .body(body)
            .build()
            .unwrap();

            // sign the request
            let priv_key = actor.get_private_key();
            let key_id = actor.get_private_key_id();
            attach_signature(
                &mut req,
                SignKeyBuilder::default()
                    .private_key(priv_key)
                    .id(key_id)
                    .build()
                    .unwrap(),
            )
            .expect("failed to attach http-signature");

            // send to the inbox
            info!("sending to inbox: {:?}", req);
            let res = client.execute(req).await.map_err(map_error)?;

            if res.status().is_success() {
                return Ok(());
            }

            tracing::warn!("Failed to send to inbox: {:?}", res);
            return Err(ServiceError::MiscError(Box::new(
                ApubReqwestError::from_response(res).await,
            )));
        }

        async fn fetch_user(
            &mut self,
            url: &str,
        ) -> Result<Actor, super::ServiceError<super::ApubFetchUserError>> {
            // TODO: sign req with maintenance user
            let req = RequestBuilder::from_parts(
                self.client(),
                Request::new(Method::GET, url.parse().unwrap()),
            )
            .header(
                "Accept",
                r#"application/ld+json; profile="https://www.w3.org/ns/activitystreams""#,
            )
            .build()
            .unwrap();

            let res = self.client().execute(req).await.map_err(map_error)?;
            let bytes = res.json::<serde_json::Value>().await.map_err(map_error)?;
            // let body = res.json::<serde_json::Value>().await.map_err(map_error)?;
            tracing::debug!("body: {:#?}", bytes);
            let person = serde_json::from_value(bytes).map_err(|e| {
                tracing::warn!("failed to parse actor: {:#?}", e);
                ServiceError::MiscError(Box::new(e))
            })?;

            Ok(person)
        }

        async fn fetch_post(
            &mut self,
            url: &str,
        ) -> Result<CreatableObject, ServiceError<ApubFetchPostError>> {
            // TODO: sign req with maintenance user
            let req = RequestBuilder::from_parts(
                self.client(),
                Request::new(Method::GET, url.parse().unwrap()),
            )
            .header(
                "Accept",
                r#"application/ld+json; profile="https://www.w3.org/ns/activitystreams""#,
            )
            .build()
            .unwrap();

            let res = self.client().execute(req).await.map_err(map_error)?;
            // let body = res.json::<serde_json::Value>().await.map_err(map_error)?;
            // debug!("body: {:#?}", body);
            let note = res.json::<CreatableObject>().await.map_err(map_error)?;

            Ok(note)
        }

        async fn fetch_webfinger(
            &mut self,
            username: &str,
            host: &str,
        ) -> Result<ApubWebfingerResponse, super::ServiceError<WebfingerError>> {
            let url = format!(
                "https://{}/.well-known/webfinger?resource=acct:{}@{}",
                host, username, host
            );
            let res = self
                .client()
                .get(url)
                .header("accept", "application/json")
                .send()
                .await
                .map_err(map_error)?;

            let json_body = res.json::<WebfingerResponse>().await.map_err(map_error)?;
            let links: Vec<_> = json_body
                .links
                .iter()
                .filter_map(
                    |l| match (l.rel.clone(), l.r#type.clone(), l.href.clone()) {
                        (Some(r), Some(t), Some(h)) => Some((r, t, h)),
                        _ => None,
                    },
                )
                .collect();

            let result = ApubWebfingerResponseBuilder::default()
                .api_url(
                    links
                        .iter()
                        .find(|link| link.0 == "self" || link.1 == "application/activity+json")
                        .map(|link| link.2.clone())
                        .ok_or(ServiceError::from_se(WebfingerError::ApiUrlNotFound))?,
                )
                .profile_url(
                    links
                        .iter()
                        .find(|link| {
                            link.0 == "http://webfinger.net/rel/profile-page"
                                || link.1 == "text/html"
                        })
                        .map(|link| link.2.clone()),
                )
                .build()
                .unwrap();
            Ok(result)
        }
    }

    // #[derive(Debug)]
    // pub enum WorkerTask {
    //     PostToInbox(PostToInboxPayload),
    //     FetchUser(GetRequestPayload),
    //     FetchPost(GetRequestPayload),
    //     FetchWebfinger(GetWebfingerPayload),
    // }

    pub struct ApubDirector {}

    impl ApubDirector {
        pub async fn prepare(conn: &lapin::Connection) -> Self {
            let chan = conn.create_channel().await.unwrap();

            chan.queue_declare(
                GET_QUEUE,
                QueueDeclareOptions::default(),
                FieldTable::default(),
            )
            .await
            .unwrap();

            chan.queue_declare(POST_QUEUE, QueueDeclareOptions::default(), {
                let mut f = FieldTable::default();
                f.insert(
                    DLX_HEADER.into(),
                    LongString::from(POST_DLX.as_bytes().to_vec()).into(),
                );
                f
            })
            .await
            .unwrap();

            chan.queue_bind(
                POST_QUEUE,
                POST_EXCHANGE,
                INBOX_POST_ROUTING_KEY,
                QueueBindOptions::default(),
                FieldTable::default(),
            )
            .await
            .unwrap();
            chan.queue_bind(
                GET_QUEUE,
                GET_REQUEST_EXCHANGE,
                FETCH_USER_ROUTING_KEY,
                QueueBindOptions::default(),
                FieldTable::default(),
            )
            .await
            .unwrap();
            chan.queue_bind(
                GET_QUEUE,
                GET_REQUEST_EXCHANGE,
                FETCH_POST_ROUTING_KEY,
                QueueBindOptions::default(),
                FieldTable::default(),
            )
            .await
            .unwrap();
            chan.queue_bind(
                GET_QUEUE,
                GET_REQUEST_EXCHANGE,
                FETCH_WEBFINGER_ROUTING_KEY,
                QueueBindOptions::default(),
                FieldTable::default(),
            )
            .await
            .unwrap();

            Self {}
        }

        pub async fn add_workers<F>(
            &mut self,
            n_workers: u32,
            conn: &lapin::Connection,
            worker_type: WorkerType,
            make_client: F,
        ) where
            F: Fn() -> ApubReqwester,
        {
            for _ in 0..n_workers {
                let chan = conn.create_channel().await.unwrap();
                let worker = ApubWorker::new(chan, make_client());
                tokio::spawn(async move {
                    worker.start(worker_type.queue_name()).await;
                });
            }
        }
    }

    #[derive(Debug, Clone, Copy)]
    pub enum WorkerType {
        PostToInbox,
        Fetcher,
    }

    impl WorkerType {
        pub fn queue_name(&self) -> &'static str {
            match self {
                WorkerType::PostToInbox => POST_QUEUE,
                WorkerType::Fetcher => GET_QUEUE,
            }
        }
    }
}

#[derive(Debug)]
pub struct QueuedApubRequester {
    chan: lapin::Channel,

    response_tx_map: Arc<Mutex<HashMap<String, tokio::sync::oneshot::Sender<Vec<u8>>>>>,
}

const POST_EXCHANGE: &str = "post";
const POST_DLX: &str = "post_dlx";
const POST_X_DELAYED: &str = "post_delayed";
const GET_REQUEST_EXCHANGE: &str = "get_request";
const RESPONSE_QUEUE: &str = "response";
const POST_DLX_QUEUE: &str = "post_dlx_queue";

pub const INBOX_POST_ROUTING_KEY: &str = "post.inbox";
pub const FETCH_USER_ROUTING_KEY: &str = "fetch.user";
pub const FETCH_POST_ROUTING_KEY: &str = "fetch.post";
pub const FETCH_WEBFINGER_ROUTING_KEY: &str = "fetch.webfinger";

pub const DLX_HEADER: &str = "x-dead-letter-exchange";
pub const DELAY_HEADER: &str = "x-delay";
pub const TTL_HEADER: &str = "x-message-ttl";
pub const MAX_RETRY_HEADER: &str = "x-max-retry";
pub const RETRY_COUNT_HEADER: &str = "x-retry-count";
const INBOX_POST_MAX_RETRY: i32 = 10;

#[derive(Debug, Clone)]
pub struct QueuedApubRequesterBuilder {
    chan: lapin::Channel,
    response_tx_map: Arc<Mutex<HashMap<String, tokio::sync::oneshot::Sender<Vec<u8>>>>>,
}

fn calculate_next_delay(current_retry: i32) -> std::time::Duration {
    use std::time::Duration;
    // delay = 2^(current_retry) + 4 s
    let delay = Duration::from_secs(2u64.pow(current_retry as u32)) + Duration::from_secs(4);
    return delay;
}

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
    pub fn new(b: QueuedApubRequesterBuilder) -> Self {
        Self {
            chan: b.chan,
            response_tx_map: b.response_tx_map,
        }
    }

    pub async fn prepare(
        conn: &lapin::Connection,
    ) -> Result<QueuedApubRequesterBuilder, lapin::Error> {
        let mut chan = conn.create_channel().await?;
        let response_tx_map = Arc::new(Mutex::new(HashMap::new()));
        Self::initialize(conn, &mut chan, response_tx_map.clone()).await?;
        Ok(QueuedApubRequesterBuilder {
            chan,
            response_tx_map,
        })
    }

    async fn initialize(
        conn: &lapin::Connection,
        chan: &mut lapin::Channel,
        response_tx_map: Arc<Mutex<HashMap<String, tokio::sync::oneshot::Sender<Vec<u8>>>>>,
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

        // Dead letter exchange and queue for POST_EXCHANGE
        chan.exchange_declare(
            POST_DLX,
            ExchangeKind::Direct,
            durable_options,
            FieldTable::default(),
        )
        .await?;
        chan.exchange_declare(
            POST_X_DELAYED,
            ExchangeKind::Custom("x-delayed-message".to_string()),
            durable_options,
            {
                let mut f = FieldTable::default();
                f.insert(
                    "x-delayed-type".into(),
                    LongString::from("topic".as_bytes().to_vec()).into(),
                );
                f
            },
        )
        .await?;
        chan.exchange_bind(
            POST_EXCHANGE,
            POST_X_DELAYED,
            "post.*",
            ExchangeBindOptions::default(),
            FieldTable::default(),
        )
        .await?;
        chan.queue_declare(
            POST_DLX_QUEUE,
            QueueDeclareOptions::default(),
            FieldTable::default(),
        )
        .await?;
        chan.queue_bind(
            POST_DLX_QUEUE,
            POST_DLX,
            INBOX_POST_ROUTING_KEY,
            QueueBindOptions::default(),
            FieldTable::default(),
        )
        .await?;
        // spawn a thread to handle messages in POST_DLX
        let post_dlx_chan = conn.create_channel().await?;
        tokio::spawn(async move {
            let mut consumer = post_dlx_chan
                .basic_consume(
                    POST_DLX_QUEUE,
                    "",
                    BasicConsumeOptions::default(),
                    FieldTable::default(),
                )
                .await
                .expect("create consumer for POST_DLX_QUEUE");
            tracing::info!("[DL WORKER] started POST_DLX_QUEUE thread");
            while let Some(msg) = consumer.next().await {
                tracing::debug!("[DL WORKER] received message from POST_DLX_QUEUE");
                match msg {
                    Err(_) => {
                        tracing::error!("failed to receive post_dlx: {:?}", msg);
                        continue;
                    }
                    Ok(msg) => {
                        let props = &msg.properties;
                        let headers = props.headers().as_ref();
                        match headers {
                            None => {
                                tracing::warn!("post_dlx received message with no retry headers");
                                report_warn(msg.reject(BasicRejectOptions { requeue: false }))
                                    .await;
                                continue;
                            }
                            Some(headers) => {
                                let current_retry = match headers.inner().get(RETRY_COUNT_HEADER) {
                                    None => {
                                        tracing::warn!("{} not set", RETRY_COUNT_HEADER);
                                        report_warn(
                                            msg.reject(BasicRejectOptions { requeue: false }),
                                        )
                                        .await;
                                        continue;
                                    }
                                    Some(v) => v.as_long_int().expect("RETRY_COUNT_HEADER is int"),
                                };
                                let max_retry = match headers.inner().get(MAX_RETRY_HEADER) {
                                    None => {
                                        tracing::warn!("{} not set", MAX_RETRY_HEADER);
                                        report_warn(
                                            msg.reject(BasicRejectOptions { requeue: false }),
                                        )
                                        .await;
                                        continue;
                                    }
                                    Some(v) => v.as_long_int().expect("MAX_RETRY_HEADER is int"),
                                };

                                if current_retry >= max_retry {
                                    // no more retry allowed
                                    tracing::warn!(
                                        "[DL WORKER] max retry reached [{}/{}], rejecting message",
                                        current_retry,
                                        max_retry
                                    );
                                    report_warn(msg.ack(BasicAckOptions::default())).await;
                                    continue;
                                }

                                // re-enqueue
                                let mut headers = headers.clone();
                                headers
                                    .insert(RETRY_COUNT_HEADER.into(), (current_retry + 1).into());
                                headers.insert(
                                    DELAY_HEADER.into(),
                                    (calculate_next_delay(current_retry).as_millis() as i32).into(),
                                );
                                match post_dlx_chan
                                    .basic_publish(
                                        POST_X_DELAYED,
                                        msg.routing_key.as_str(),
                                        BasicPublishOptions::default(),
                                        &msg.data,
                                        BasicProperties::default().with_headers(headers),
                                    )
                                    .await
                                {
                                    Ok(_) => {
                                        report_warn(msg.ack(BasicAckOptions::default())).await;
                                        tracing::debug!(
                                            "[DL WORKER] re-enqueued message with delay [{}/{}]",
                                            current_retry,
                                            max_retry
                                        );
                                    }
                                    Err(e) => {
                                        tracing::error!(
                                            "[DL WORKER] failed to re-enqueue message [{}/{}]: {:?}", current_retry, max_retry,
                                            e
                                        );
                                        report_warn(msg.reject(BasicRejectOptions::default()))
                                            .await;
                                    }
                                }
                            }
                        }
                    }
                }
            }
        });

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

        // RPC receiver
        let mut get_request_response_consumer = chan
            .basic_consume(
                RESPONSE_QUEUE,
                "",
                BasicConsumeOptions::default(),
                FieldTable::default(),
            )
            .await?;
        let response_tx_map = response_tx_map.clone();
        // spawn a thread to handle RPC responses
        tokio::spawn(async move {
            while let Some(delivery) = get_request_response_consumer.next().await {
                let delivery = match delivery {
                    Ok(d) => d,
                    Err(e) => {
                        tracing::error!("failed to receive RPC response: {:?}", e);
                        continue;
                    }
                };
                match delivery.ack(BasicAckOptions::default()).await {
                    Ok(_) => {}
                    Err(e) => {
                        tracing::error!("failed to ack RPC response: {:?}", e);
                        continue;
                    }
                };
                let id = delivery
                    .properties
                    .correlation_id()
                    .as_ref()
                    .map(|id| id.to_string())
                    .expect("correlation_id not set");

                let mut map = response_tx_map.lock().unwrap();
                // debug
                tracing::debug!("dumping response tx map");
                for (k, _) in map.iter() {
                    tracing::debug!("response tx: {:?}", k);
                }
                tracing::debug!("dumping response tx map end");
                let tx = map.remove(&id);
                if let Some(tx) = tx {
                    tx.send(delivery.data).expect("send response");
                } else {
                    tracing::warn!("response tx not found for corr_id: {:?}", id);
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
        let timeout = std::time::Duration::from_secs(5);

        let mut headers = FieldTable::default();
        headers.insert(TTL_HEADER.into(), (timeout.as_millis() as i32).into());

        let (tx, rx) = tokio::sync::oneshot::channel();
        self.response_tx_map
            .lock()
            .unwrap()
            .insert(response_id.clone(), tx);

        self.chan
            .basic_publish(
                GET_REQUEST_EXCHANGE,
                FETCH_USER_ROUTING_KEY,
                BasicPublishOptions::default(),
                &encode_payload(payload),
                BasicProperties::default()
                    .with_reply_to(RESPONSE_QUEUE.into())
                    .with_correlation_id(response_id.clone().into())
                    .with_headers(headers),
            )
            .await?;

        let result = wait_for_result(rx, timeout).await;
        self.response_tx_map.lock().unwrap().remove(&response_id);
        result
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
        let timeout = std::time::Duration::from_secs(5);

        let mut headers = FieldTable::default();
        headers.insert(TTL_HEADER.into(), (timeout.as_millis() as i32).into());

        let (tx, rx) = tokio::sync::oneshot::channel();
        self.response_tx_map
            .lock()
            .unwrap()
            .insert(response_id.clone(), tx);

        self.chan
            .basic_publish(
                GET_REQUEST_EXCHANGE,
                FETCH_WEBFINGER_ROUTING_KEY,
                BasicPublishOptions::default(),
                &encode_payload(payload),
                BasicProperties::default()
                    .with_reply_to(RESPONSE_QUEUE.into())
                    .with_correlation_id(response_id.clone().into())
                    .with_headers(headers),
            )
            .await?;

        let result = wait_for_result(rx, timeout).await;
        self.response_tx_map.lock().unwrap().remove(&response_id);
        result
    }

    async fn fetch_post(
        &mut self,
        url: &str,
    ) -> Result<CreatableObject, ServiceError<ApubFetchPostError>> {
        let payload = GetRequestPayload {
            url: url.to_string(),
        };
        let response_id = Self::generate_random_id();
        let timeout = std::time::Duration::from_secs(5);

        let mut headers = FieldTable::default();
        headers.insert(TTL_HEADER.into(), (timeout.as_millis() as i32).into());

        let (tx, rx) = tokio::sync::oneshot::channel();
        self.response_tx_map
            .lock()
            .unwrap()
            .insert(response_id.clone(), tx);

        self.chan
            .basic_publish(
                GET_REQUEST_EXCHANGE,
                FETCH_POST_ROUTING_KEY,
                BasicPublishOptions::default(),
                &encode_payload(payload),
                BasicProperties::default()
                    .with_reply_to(RESPONSE_QUEUE.into())
                    .with_correlation_id(response_id.clone().into())
                    .with_headers(headers),
            )
            .await?;

        let result = wait_for_result(rx, timeout).await;
        self.response_tx_map.lock().unwrap().remove(&response_id);
        result
    }
}
