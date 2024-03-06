use derive_builder::Builder;
use derive_getters::Getters;
use reqwest::{
    header::{self, HeaderMap, HeaderValue},
    IntoUrl,
};
use serde::Deserialize;
use tracing::warn;

use crate::{
    models::ApubWebfingerResponseBuilder,
    services::{ServiceError, WebfingerError},
};

use super::{ApubRequestService, MiscError};

#[derive(Debug, Builder)]
pub struct ApubReqwestConfig {
    timeout: std::time::Duration,
}

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
                .build()
                .expect("failed to build reqwest client"),
        }
    }
}

#[derive(Debug)]
pub struct ApubReqwest {
    client: ApubReqwester,
    config: ApubReqwestConfig,
}

impl ApubReqwest {
    pub fn new(client: ApubReqwester, config: ApubReqwestConfig) -> Self {
        Self { client, config }
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

impl ApubRequestService for ApubReqwest {
    async fn post_to_inbox(
        &mut self,
        url: impl IntoUrl,
        activity: &crate::models::ApubActivity,
        actor: impl Into<crate::models::ApubActor>,
    ) -> Result<(), super::ServiceError<super::PostToInboxError>> {
        let body = activity.to_json();

        let client = self.client();
        let actor = actor.into();

        // TODO: sign the request

        // send to the inbox
        let res = client
            .post(url)
            .header("Content-Type", "application/activity+json")
            .header("Accept", "application/activity+json")
            .body(body)
            .send()
            .await?;

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
        url: impl IntoUrl,
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
            .await?;

        let json_body = res.json::<WebfingerResponse>().await?;

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
