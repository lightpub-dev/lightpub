use futures::StreamExt;
use lapin::{
    options::{
        BasicAckOptions, BasicPublishOptions, BasicRejectOptions, ExchangeDeclareOptions,
        QueueBindOptions, QueueDeclareOptions,
    },
    types::{FieldTable, LongString},
    BasicProperties, ExchangeKind,
};
use reqwest::{Method, Request, RequestBuilder};
use rsa::RsaPrivateKey;
use tracing::{info, warn};

use lightpub_backend::{
    apub::{ApubReqwestError, ApubReqwestErrorBuilder, ApubReqwester, WebfingerResponse},
    holder, ApubFetchPostError, ApubFetchUserError, Holder, PostToInboxError, ServiceError,
    WebfingerError,
};
use lightpub_model::{
    apub::{context::ContextAttachable, Activity, Actor, CreatableObject},
    ApubSigner, ApubWebfingerResponse, ApubWebfingerResponseBuilder,
};
use lightpub_utils::key::{attach_signature, SignKeyBuilder};

use lightpub_backend::apub::queue::{
    transport::{
        decode_payload, encode_payload, GetRequestPayload, GetWebfingerPayload, PostToInboxPayload,
        ResponsePayload,
    },
    DLX_HEADER, FETCH_POST_ROUTING_KEY, FETCH_USER_ROUTING_KEY, FETCH_WEBFINGER_ROUTING_KEY,
    GET_REQUEST_EXCHANGE, INBOX_POST_ROUTING_KEY, POST_DLX, POST_EXCHANGE,
};

fn map_error<T>(e: reqwest::Error) -> ServiceError<T> {
    warn!("reqwest error: {:#?}", e);
    ServiceError::MiscError(Box::new(
        ApubReqwestErrorBuilder::default()
            .status(
                e.status()
                    .unwrap_or(reqwest::StatusCode::INTERNAL_SERVER_ERROR),
            )
            .body(e.to_string())
            .build()
            .unwrap(),
    ))
}

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
                        let response: ResponsePayload<Actor, ApubFetchUserError> = result.into();
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
                        let result = self.fetch_webfinger(&payload.username, &payload.host).await;
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
    ) -> Result<(), ServiceError<PostToInboxError>> {
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

    async fn fetch_user(&mut self, url: &str) -> Result<Actor, ServiceError<ApubFetchUserError>> {
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
    ) -> Result<ApubWebfingerResponse, ServiceError<WebfingerError>> {
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
                        link.0 == "http://webfinger.net/rel/profile-page" || link.1 == "text/html"
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

        let mut durable_options = ExchangeDeclareOptions::default();
        durable_options.durable = true;
        chan.exchange_declare(
            POST_EXCHANGE,
            ExchangeKind::Direct,
            durable_options,
            FieldTable::default(),
        )
        .await
        .unwrap();

        let default_options = ExchangeDeclareOptions::default();
        chan.exchange_declare(
            GET_REQUEST_EXCHANGE,
            ExchangeKind::Direct,
            default_options,
            FieldTable::default(),
        )
        .await
        .unwrap();

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
