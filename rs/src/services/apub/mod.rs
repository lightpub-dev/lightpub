use derive_builder::Builder;
use derive_getters::Getters;
use reqwest::{
    header::{self, HeaderMap, HeaderValue},
    Method, Request, RequestBuilder, Url,
};
use serde::{Deserialize, Serialize};
use sqlx::MySqlPool;
use tracing::{info, warn};

use uuid::fmt::Simple;

use crate::{
    models::{
        ApubFollowBuilder, ApubPayload, ApubPerson, ApubSigner, ApubWebfingerResponseBuilder,
        HasRemoteUri,
    },
    services::{ServiceError, WebfingerError},
    utils::key::{attach_signature, SignKeyBuilder},
};

use super::{
    id::IDGetterService, ApubFollowError, ApubFollowService, ApubRequestService, MiscError,
};

pub mod queue;
pub mod render;
#[derive(Debug, Clone)]
pub struct ApubReqwester {
    client: reqwest::Client,
}

impl ApubReqwester {
    pub fn new() -> Self {
        let no_ssl_verify = true;
        let mut headers = HeaderMap::new();
        headers.insert(header::USER_AGENT, HeaderValue::from_static("lightpub/0.1"));
        Self {
            client: reqwest::ClientBuilder::new()
                .danger_accept_invalid_certs(no_ssl_verify)
                .default_headers(headers)
                .timeout(std::time::Duration::from_secs(10)) // TODO: make this configurable
                .build()
                .expect("failed to build reqwest client"),
        }
    }
}

pub fn new_apub_reqwester_service() -> ApubReqwest {
    ApubReqwest {
        client: ApubReqwester::new(),
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

impl ApubRequestService for ApubReqwest {
    async fn post_to_inbox<T: Serialize>(
        &mut self,
        url: impl Into<Url>,
        activity: &ApubPayload<T>,
        actor: &impl ApubSigner,
    ) -> Result<(), super::ServiceError<super::PostToInboxError>> {
        let body = activity.to_json();
        info!("body: {:?}", &body);

        let client = self.client();

        let mut req =
            RequestBuilder::from_parts(self.client(), Request::new(Method::POST, url.into()))
                .header("Content-Type", "application/activity+json")
                // .header("Accept", "application/activity+json")
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

        warn!("Failed to send to inbox: {:?}", res);
        return Err(ServiceError::MiscError(Box::new(
            ApubReqwestError::from_response(res).await,
        )));
    }

    async fn fetch_user(
        &mut self,
        url: impl Into<Url>,
    ) -> Result<crate::models::ApubPerson, super::ServiceError<super::ApubFetchUserError>> {
        let url = url.into();
        // TODO: sign req with maintenance user
        let req = RequestBuilder::from_parts(self.client(), Request::new(Method::GET, url))
            .header("Accept", "application/activity+json")
            .build()
            .unwrap();

        let res = self.client().execute(req).await.map_err(map_error)?;
        let person = res.json::<ApubPerson>().await.map_err(map_error)?;

        Ok(person)
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

#[derive(Debug, Deserialize)]
struct WebfingerResponse {
    #[allow(unused)]
    subject: String,
    links: Vec<WebfingerLinks>,
}

#[derive(Debug, Deserialize)]
struct WebfingerLinks {
    href: Option<String>,
    rel: Option<String>,
    r#type: Option<String>,
}

#[derive(Debug, Clone)]
pub struct DBApubFollowService {
    pool: MySqlPool,
    id_getter: IDGetterService,
}

pub fn new_apub_follow_service(pool: MySqlPool, id_getter: IDGetterService) -> DBApubFollowService {
    DBApubFollowService { pool, id_getter }
}

#[derive(Debug)]
struct UserFollowInfo {
    req_id: Simple,
    follower_id: Simple,
    follower_uri: Option<String>,
    followee_id: Simple,
    followee_uri: Option<String>,
}

#[derive(Debug)]
struct UserFollowUser {
    id: Simple,
    uri: Option<String>,
}

impl HasRemoteUri for UserFollowUser {
    fn get_local_id(&self) -> String {
        self.id.to_string()
    }

    fn get_remote_uri(&self) -> Option<String> {
        self.uri.clone()
    }
}

impl HasRemoteUri for UserFollowInfo {
    fn get_local_id(&self) -> String {
        self.req_id.to_string()
    }

    fn get_remote_uri(&self) -> Option<String> {
        None
    }
}

impl ApubFollowService for DBApubFollowService {
    async fn create_follow_accept(
        &mut self,
        _follow_req_id: uuid::Uuid,
    ) -> Result<crate::models::ApubAccept, ServiceError<ApubFollowError>> {
        todo!()
    }

    async fn create_follow_request(
        &mut self,
        follow_req_id: uuid::Uuid,
    ) -> Result<crate::models::ApubFollow, ServiceError<ApubFollowError>> {
        let uf = sqlx::query_as!(UserFollowInfo, r#"
        SELECT r.id AS `req_id: Simple`, r.follower_id AS `follower_id: Simple`, u1.uri AS follower_uri, r.followee_id AS `followee_id: Simple`, u2.uri AS followee_uri
        FROM user_follow_requests AS r
        INNER JOIN users u1 ON r.follower_id = u1.id
        INNER JOIN users u2 ON r.followee_id = u2.id
        WHERE r.id = ? AND r.uri IS NULL AND r.incoming = 0
        "#, follow_req_id.simple().to_string()).fetch_optional(&self.pool).await?;
        let uf = match uf {
            None => return Err(ServiceError::from_se(ApubFollowError::RequestNotFound)),
            Some(uf) => uf,
        };
        let uf_id = self.id_getter.get_follower_request_id(&uf);

        let follower_id = self.id_getter.get_user_id(&UserFollowUser {
            id: uf.follower_id,
            uri: uf.follower_uri.clone(),
        });
        let followee_id = self.id_getter.get_user_id(&UserFollowUser {
            id: uf.followee_id,
            uri: uf.followee_uri.clone(),
        });

        Ok(ApubFollowBuilder::default()
            .id(uf_id)
            .actor(follower_id.into())
            .object(followee_id.into())
            .build()
            .unwrap())
    }
}
