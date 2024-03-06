use derive_builder::Builder;
use derive_getters::Getters;
use reqwest::{
    header::{self, HeaderMap, HeaderValue},
    IntoUrl, Method, Request, RequestBuilder, Url,
};
use serde::Deserialize;
use tracing::warn;

use crate::{
    models::{ApubSigner, ApubWebfingerResponseBuilder},
    services::{ServiceError, WebfingerError},
    utils::key::{attach_signature, SignKeyBuilder},
};

use super::{ApubRequestService, MiscError};

pub mod render;
#[derive(Debug, Clone)]
pub struct ApubReqwester {
    client: reqwest::Client,
}

impl ApubReqwester {
    pub fn new() -> Self {
        let mut headers = HeaderMap::new();
        headers.insert(header::USER_AGENT, HeaderValue::from_static("lightpub/0.1"));
        Self {
            client: reqwest::ClientBuilder::new()
                .default_headers(headers)
                .timeout(std::time::Duration::from_secs(10)) // TODO: make this configurable
                .build()
                .expect("failed to build reqwest client"),
        }
    }
}

#[derive(Debug)]
pub struct ApubReqwest {
    client: ApubReqwester,
}

impl ApubReqwest {
    pub fn new(client: ApubReqwester) -> Self {
        Self { client }
    }

    fn client(&self) -> reqwest::Client {
        self.client.client.clone()
    }
}

#[derive(Debug, Builder, Getters)]
pub struct ApubReqwestError {
    status: reqwest::StatusCode,
    body: String,
}

impl MiscError for ApubReqwestError {
    fn message(&self) -> &str {
        "internal server error"
    }

    fn status_code(&self) -> i32 {
        500
    }
}

impl ApubReqwestError {
    pub async fn from_response(res: reqwest::Response) -> Self {
        let status = res.status();
        let body = res.text().await.unwrap_or_else(|_| "no body".to_string());
        Self { status, body }
    }
}

fn map_error<T>(e: reqwest::Error) -> ServiceError<T> {
    ServiceError::MiscError(Box::new(
        ApubReqwestErrorBuilder::default().build().unwrap(),
    ))
}

impl ApubRequestService for ApubReqwest {
    async fn post_to_inbox(
        &mut self,
        url: impl Into<Url>,
        activity: &crate::models::ApubActivity,
        actor: &impl ApubSigner,
    ) -> Result<(), super::ServiceError<super::PostToInboxError>> {
        let body = activity.to_json();

        let client = self.client();
        let actor_id = actor.get_user_id();

        let mut req =
            RequestBuilder::from_parts(self.client(), Request::new(Method::POST, url.into()))
                .header("Content-Type", "application/activity+json")
                .header("Accept", "application/activity+json")
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
        let res = client.execute(req).await.map_err(map_error)?;

        if res.status().is_success() {
            return Ok(());
        }

        warn!("Failed to send to inbox: {:?}", res);
        return Err(ServiceError::MiscError(Box::new(
            ApubReqwestError::from_response(res).await,
        )));
    }

    async fn fetch_user(
        &mut self,
        url: impl Into<Url>,
    ) -> Result<crate::models::ApubPerson, super::ServiceError<super::ApubFetchUserError>> {
        todo!()
    }

    async fn fetch_webfinger(
        &mut self,
        username: &str,
        host: &str,
    ) -> Result<crate::models::ApubWebfingerResponse, super::ServiceError<super::WebfingerError>>
    {
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

        let result = ApubWebfingerResponseBuilder::default()
            .api_url(
                json_body
                    .links
                    .iter()
                    .find(|link| link.rel == "self" || link.r#type == "application/activity+json")
                    .map(|link| link.href.clone())
                    .ok_or(ServiceError::from_se(WebfingerError::ApiUrlNotFound))?,
            )
            .profile_url(
                json_body
                    .links
                    .iter()
                    .find(|link| {
                        link.rel == "http://webfinger.net/rel/profile-page"
                            || link.r#type == "text/html"
                    })
                    .map(|link| link.href.clone()),
            )
            .build()
            .unwrap();
        Ok(result)
    }
}

#[derive(Debug, Deserialize)]
struct WebfingerResponse {
    subject: String,
    links: Vec<WebfingerLinks>,
}

#[derive(Debug, Deserialize)]
struct WebfingerLinks {
    href: String,
    rel: String,
    r#type: String,
}
