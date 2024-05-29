use anyhow::anyhow;
use async_trait::async_trait;
use derive_builder::Builder;
use derive_getters::Getters;
use reqwest::{
    header::{self, HeaderMap, HeaderValue},
    Method, Request, RequestBuilder,
};
use serde::Deserialize;
use sqlx::SqlitePool;
use tracing::{debug, info, warn};

use lightpub_model::{
    apub::{
        context::ContextAttachable, AcceptActivity, AcceptActivityBuilder, AcceptableActivity,
        Activity, Actor, CreatableObject, FollowActivity, FollowActivityBuilder, IdOrObject,
        UndoActivity, UndoActivityBuilder, UndoableActivity,
    },
    ApubSigner, ApubWebfingerResponseBuilder, HasRemoteUri,
};
use lightpub_model::{ApubWebfingerResponse, UserSpecifier};
use lightpub_utils::key::{attach_signature, SignKeyBuilder};
use uuid::fmt::Simple;

use crate::{holder, ServiceError, WebfingerError};
use lightpub_config::Config;

use self::{
    dummy::DummyRequester,
    queue::{QueuedApubRequester, QueuedApubRequesterBuilder},
    render::ApubRendererService,
};

use super::{
    id::IDGetterService, AllUserFinderService, ApubFetchPostError, ApubFollowService,
    ApubRequestService, Holder, MiscError,
};

pub mod db_queue;
pub mod dummy;
pub mod inbox;
pub mod post;
pub mod queue;
pub mod render;

#[derive(Debug, Clone)]
pub struct ApubReqwester {
    pub client: reqwest::Client,
}

impl ApubReqwester {
    pub fn new(config: &Config) -> Self {
        let no_ssl_verify = !config.dev.ssl_verify;
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

pub fn new_apub_reqwester_service(
    conn: QueuedApubRequesterBuilder,
    config: &Config,
) -> holder!(ApubRequestService) {
    if config.federation.enabled {
        // Holder::new(ApubReqwest {
        //     client: ApubReqwester::new(config),
        // })
        Holder::new(QueuedApubRequester::new(conn))
    } else {
        Holder::new(DummyRequester::new())
    }
}

#[derive(Debug)]
pub struct ApubReqwest {
    pub client: ApubReqwester,
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

#[async_trait]
impl ApubRequestService for ApubReqwest {
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

        warn!("Failed to send to inbox: {:?}", res);
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
        debug!("body: {:#?}", bytes);
        let person = serde_json::from_value(bytes).map_err(|e| {
            warn!("failed to parse actor: {:#?}", e);
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
    ) -> Result<ApubWebfingerResponse, super::ServiceError<super::WebfingerError>> {
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
pub struct WebfingerResponse {
    #[allow(unused)]
    pub subject: String,
    pub links: Vec<WebfingerLinks>,
}

#[derive(Debug, Deserialize)]
pub struct WebfingerLinks {
    pub href: Option<String>,
    pub rel: Option<String>,
    pub r#type: Option<String>,
}

pub struct DBApubFollowService {
    pool: SqlitePool,
    id_getter: IDGetterService,
    user_finder: holder!(AllUserFinderService),
}

pub fn new_apub_follow_service(
    pool: SqlitePool,
    id_getter: IDGetterService,
    user_finder: holder!(AllUserFinderService),
) -> holder!(ApubFollowService) {
    Holder::new(DBApubFollowService {
        pool,
        id_getter,
        user_finder,
    })
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
struct UserFollowInfoWithUri {
    follower_id: Simple,
    follower_uri: Option<String>,
    followee_id: Simple,
    followee_uri: Option<String>,
    req_uri: String,
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

#[async_trait]
impl ApubFollowService for DBApubFollowService {
    async fn create_follow_accept(
        &mut self,
        follow_req_id: uuid::Uuid,
    ) -> Result<AcceptActivity, anyhow::Error> {
        let follow_req_id_str = follow_req_id.simple().to_string();
        let uf = sqlx::query_as!(UserFollowInfoWithUri, r#"
        SELECT r.follower_id AS `follower_id: Simple`, u1.uri AS follower_uri, r.followee_id AS `followee_id: Simple`, u2.uri AS followee_uri, r.uri AS `req_uri!`
        FROM user_follow_requests AS r
        INNER JOIN users u1 ON r.follower_id = u1.id
        INNER JOIN users u2 ON r.followee_id = u2.id
        WHERE r.id = ? AND r.incoming = 1
        "#, follow_req_id_str).fetch_optional(&self.pool).await?;
        let uf = match uf {
            None => return Err(anyhow!("follow request not found")),
            Some(uf) => uf,
        };

        let follower_id = self.id_getter.get_user_id(&UserFollowUser {
            id: uf.follower_id,
            uri: uf.follower_uri.clone(),
        });
        let followee_id = self.id_getter.get_user_id(&UserFollowUser {
            id: uf.followee_id,
            uri: uf.followee_uri.clone(),
        });

        /*
        Accept {
            actor: followee_id
            object: {
                id: follow_req_id
                actor: follower_id
                object: followee_id
            }
        }
        */
        let accept = AcceptActivityBuilder::default()
            .id(None)
            .actor(followee_id.clone())
            .object(IdOrObject::Object(AcceptableActivity::Follow(
                FollowActivityBuilder::default()
                    .id(Some(uf.req_uri))
                    .actor(follower_id)
                    .object(IdOrObject::Id(followee_id))
                    .build()
                    .unwrap(),
            )))
            .build()
            .unwrap();
        Ok(accept)
    }

    async fn create_follow_request(
        &mut self,
        follow_req_id: uuid::Uuid,
    ) -> Result<FollowActivity, anyhow::Error> {
        let follow_req_id_str = follow_req_id.simple().to_string();
        let uf = sqlx::query_as!(UserFollowInfo, r#"
        SELECT r.id AS `req_id: Simple`, r.follower_id AS `follower_id: Simple`, u1.uri AS follower_uri, r.followee_id AS `followee_id: Simple`, u2.uri AS followee_uri
        FROM user_follow_requests AS r
        INNER JOIN users u1 ON r.follower_id = u1.id
        INNER JOIN users u2 ON r.followee_id = u2.id
        WHERE r.id = ? AND r.uri IS NULL AND r.incoming = 0
        "#, follow_req_id_str).fetch_optional(&self.pool).await?;
        let uf = match uf {
            None => return Err(anyhow!("follow request not found")),
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

        let follow = FollowActivityBuilder::default()
            .id(Some(uf_id))
            .actor(follower_id)
            .object(IdOrObject::Id(followee_id))
            .build()
            .unwrap();
        Ok(follow)
    }

    async fn create_unfollow_request(
        &mut self,
        follower_id: &UserSpecifier,
        followee_id: &UserSpecifier,
    ) -> Result<UndoActivity, anyhow::Error> {
        let follower = self.user_finder.find_user_by_specifier(follower_id).await?;
        let followee = self.user_finder.find_user_by_specifier(followee_id).await?;

        let follower_id = self.id_getter.get_user_id(&UserFollowUser {
            id: follower.id,
            uri: follower.uri,
        });
        let followee_id = self.id_getter.get_user_id(&UserFollowUser {
            id: followee.id,
            uri: followee.uri,
        });

        let unfollow = UndoActivityBuilder::default()
            .id(None)
            .actor(follower_id.clone())
            .object(UndoableActivity::Follow(
                FollowActivityBuilder::default()
                    .id(None)
                    .actor(follower_id)
                    .object(IdOrObject::Id(followee_id))
                    .build()
                    .unwrap(),
            ))
            .build()
            .unwrap();
        Ok(unfollow)
    }
}

pub fn new_apub_renderer_service(config: Config) -> ApubRendererService {
    ApubRendererService::new(IDGetterService::new(config))
}
