use async_trait::async_trait;
use lightpub_model::{
    apub::{context::ContextAttachable, Activity, Actor, CreatableObject},
    ApubWebfingerResponse, ApubWebfingerResponseBuilder,
};
use lightpub_utils::key::{attach_signature, SignKeyBuilder};
use reqwest::{Method, Request, RequestBuilder};
use sqlx::SqlitePool;
use tracing::{info, warn};

use super::{ApubReqwester, WebfingerResponse};
use crate::{
    apub::{ApubReqwestError, ApubReqwestErrorBuilder},
    holder, ApubFetchPostError, ApubFetchUserError, ApubRequestService, ApubSigner, Holder,
    PostToInboxError, ServiceError, WebfingerError,
};

use lightpub_model::queue::{PostToInboxPayload, SignerPayload, WorkerTask};

pub struct ApubQueueService {
    pool: SqlitePool,
    client: ApubReqwester,
}

#[async_trait]
impl ApubRequestService for ApubQueueService {
    async fn post_to_inbox(
        &mut self,
        url: &str,
        activity: &Activity,
        actor: holder!(ApubSigner),
    ) -> Result<(), ServiceError<PostToInboxError>> {
        let user_id = actor.get_user_id();
        let private_key = actor.get_private_key();
        let private_key_id = actor.get_private_key_id();
        let payload = WorkerTask::PostToInbox(PostToInboxPayload {
            url: url.to_string(),
            activity: activity.clone(),
            actor: SignerPayload {
                user_id,
                private_key: private_key.clone(),
                private_key_id: private_key_id.clone(),
            },
        });
        let payload_json = serde_json::to_string(&payload).unwrap();

        sqlx::query!(
            r#"
        INSERT INTO QueuedTask(started_at, max_retry, payload) VALUES(NULL, 3, ?)
        "#,
            payload_json
        )
        .execute(&self.pool)
        .await
        .map_err(|e| {
            warn!("failed to insert task: {:#?}", e);
            ServiceError::MiscError(Box::new(e))
        })?;

        Ok(())
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

impl ApubQueueService {
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
}

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
