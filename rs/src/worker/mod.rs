use reqwest::{Method, Request, RequestBuilder};
use rsa::RsaPrivateKey;
use sqlx::{Pool, Sqlite};
use tracing::{info, warn};

use crate::backend::{
    apub::{ApubReqwestError, ApubReqwestErrorBuilder, ApubReqwester},
    PostToInboxError, ServiceError,
};
use crate::holder;
use crate::model::{
    apub::{context::ContextAttachable, Activity},
    ApubSigner,
};
use crate::utils::key::{attach_signature, SignKeyBuilder};

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
    current_retry: i64,
    max_retry: i64,
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
            .fetch_optional(&mut *tx)
            .await?;

            if let Some(task) = task {
                sqlx::query!(
                    "UPDATE QueuedTask SET started_at = (DATETIME('now')) WHERE id = ?",
                    task.id
                )
                .execute(&self.pool)
                .await?;
            } else {
                tx.commit().await?;
                tokio::time::sleep(std::time::Duration::from_secs(1)).await;
            }
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

pub struct ApubDirector<F> {
    pool: Pool<Sqlite>,
    client_maker: F,
}

impl<F> ApubDirector<F>
where
    F: Fn() -> ApubReqwester,
{
    pub fn new(pool: Pool<Sqlite>, client_maker: F) -> Self {
        Self { pool, client_maker }
    }

    pub async fn start_workers(&mut self) {}

    pub async fn add_workers(&mut self) {
        let mut worker = ApubWorker::new(self.pool.clone(), (self.client_maker)());
        tokio::spawn(async move {
            worker.start().await;
        });
    }
}
