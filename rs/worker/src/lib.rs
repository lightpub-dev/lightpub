use futures::StreamExt;
use reqwest::{Method, Request, RequestBuilder};
use rsa::RsaPrivateKey;
use sqlx::{Pool, Sqlite};
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

pub struct ApubWorker {
    pool: Pool<Sqlite>,
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

#[derive(Debug)]
struct QueuedTask {
    id: i64,
    created_at: chrono::NaiveDateTime,
    started_at: Option<chrono::NaiveDateTime>,
    current_entry: i32,
    max_retry: i32,
    payload: String,
}

impl ApubWorker {
    pub fn new(pool: Pool<Sqlite>, client: ApubReqwester) -> Self {
        Self { pool, client }
    }

    fn client(&self) -> reqwest::Client {
        self.client.client.clone()
    }

    pub async fn start(&mut self) -> Result<(), anyhow::Error> {
        loop {
            let mut tx = self.pool.begin().await?;
            let task = sqlx::query_as!(
                QueuedTask,
                "SELECT id, current_retry, max_retry, payload FROM QueuedTask ORDER BY id ASC LIMIT 1"
            )
            .fetch_optional(&mut tx)
            .await?;
        }
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
}

pub struct ApubDirector<F>
where
    F: Fn() -> ApubReqwester,
{
    pool: Pool<Sqlite>,
    client_maker: F,
}

impl ApubDirector {
    pub fn new(pool: Pool<Sqlite>, client_maker: F) -> Self {
        Self { pool, client_maker }
    }

    pub async fn start_workers(&mut self) {}

    pub async fn add_workers<F>(&mut self)
    where
        F: Fn() -> ApubReqwester,
    {
        let worker = ApubWorker::new(pool, self.client_maker());
        tokio::spawn(async move {
            worker.start().await;
        });
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
