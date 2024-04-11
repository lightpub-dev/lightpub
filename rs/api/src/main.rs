use lightpub_backend::apub::queue::QueuedApubRequester;
use lightpub_backend::db::new_db_user_post_service;
use lightpub_model::apub::context::ContextAttachable;
use lightpub_model::apub::{
    AcceptableActivity, Actor, CreatableObject, HasId, IdOrObject, RejectableActivity,
    UndoableActivity, PUBLIC,
};
use lightpub_model::reaction::Reaction;
use lightpub_model::{PostSpecifier, UserSpecifier};

use actix_cors::Cors;
use actix_multipart::form::MultipartForm;
use actix_web::http::header;
use actix_web::{
    delete, get, middleware::Logger, post, put, web, App, FromRequest, HttpResponse, HttpServer,
    Responder,
};
use clap::Parser;
use lapin::ConnectionProperties;
use lightpub_api::state::AppState;
use lightpub_backend::apub::render::RenderedNoteObject;
use lightpub_backend::db::{new_all_user_finder_service, new_db_key_fetcher_service};
use lightpub_backend::db::{new_db_file_upload_service, new_db_user_profile_service};
use lightpub_backend::{
    apub::{new_apub_renderer_service, new_apub_reqwester_service},
    db::{new_follow_service, new_post_create_service},
    FollowRequestSpecifier, IncomingFollowRequest, PostCreateError, PostCreateRequest,
    PostCreateRequestNormalBuilder, PostCreateRequestQuoteBuilder, PostCreateRequestReplyBuilder,
    PostCreateRequestRepostBuilder,
};
use lightpub_backend::{
    db::{new_auth_service, new_local_user_finder_service},
    id::IDGetterService,
    AuthError, LocalUserFindError, ServiceError, UserCreateRequest, UserCreateRequestBuilder,
    UserLoginError, UserLoginRequest, UserLoginRequestBuilder,
};
use lightpub_backend::{
    FetchFollowListOptions, FetchUserPostsOptions, PostInteractionAction, TimelineOptions,
    UserCreateError, UserProfileUpdate,
};
use lightpub_backend::{PostDeleteError, PostFetchError};
use lightpub_config::Config;
use lightpub_model::apub::LikeActivity;
use lightpub_model::http::{HeaderMapWrapper, Method};
use lightpub_model::pagination::{
    CollectionPageResponse, CollectionPageType, CollectionResponse, CollectionType,
    PaginatableWrapper, PaginatedResponse,
};
use lightpub_model::reaction::ReactionError;
use lightpub_model::{PostPrivacy, User};
use lightpub_utils::generate_uuid;
use lightpub_utils::key::VerifyError;
use lightpub_utils::key::{verify_signature, KeyFetcher};
use serde::{Deserialize, Serialize};
use serde_json::json;
use sqlx::mysql::MySqlPoolOptions;
use std::borrow::BorrowMut;
use std::path::PathBuf;
use std::time::Duration;
use std::{
    fmt::{Debug, Display, Formatter},
    future::Future,
    io::Read,
    pin::Pin,
};
use tracing::{debug, error, info, warn};
use uuid::{fmt::Simple, Uuid};

use lightpub_backend::db::new_user_service;

#[derive(Debug)]
struct AuthUser {
    authed_user: Option<User>,
}

impl AuthUser {
    pub fn from_user(user: User) -> Self {
        Self {
            authed_user: Some(user),
        }
    }

    pub fn unauthed() -> Self {
        Self { authed_user: None }
    }

    pub fn must_auth(&self) -> Result<&User, ErrorResponse> {
        match &self.authed_user {
            Some(u) => Ok(&u),
            None => Err(ErrorResponse::new_status(401, "unauthorized")),
        }
    }

    pub fn may_auth(&self) -> Result<&Option<User>, ErrorResponse> {
        Ok(&self.authed_user)
    }
}

type HandlerResponse<T> = Result<T, ErrorResponse>;

impl FromRequest for AuthUser {
    type Error = ErrorResponse;
    type Future = Pin<Box<dyn Future<Output = Result<AuthUser, Self::Error>>>>;

    fn from_request(
        req: &actix_web::HttpRequest,
        _payload: &mut actix_web::dev::Payload,
    ) -> Self::Future {
        let req = req.clone();
        Box::pin(async move {
            let data = web::Data::<AppState>::extract(&req).await.unwrap();

            // try with http-signature auth
            let mut key_fetcher = new_db_key_fetcher_service(
                data.pool().clone(),
                data.queue().clone(),
                data.config().clone(),
            );
            let sig_result = verify_signature(
                HeaderMapWrapper::from_actix(req.headers()),
                Method::from_actix(req.method()),
                req.path(),
                key_fetcher.borrow_mut() as &mut (dyn KeyFetcher + Send + Sync),
            )
            .await;
            match sig_result {
                Ok(u) => {
                    let mut user_finder = new_all_user_finder_service(
                        data.pool().clone(),
                        data.queue().clone(),
                        data.config().clone(),
                    );
                    let user = user_finder.find_user_by_specifier(&u).await?;
                    return Ok(AuthUser::from_user(user));
                }
                Err(e) => {
                    use VerifyError::*;
                    match e {
                        SignatureNotFound => {
                            // try with bearer token, continue
                        }
                        SignatureInvalid | SignatureNotMatch | KeyNotFound | InsufficientHeader => {
                            return Err(ErrorResponse::new_status(401, "unauthorized"))
                        }
                        Other(_) => {
                            error!("Other error: {:?}", e);
                            return Err(ErrorResponse::new_status(500, "internal server error"));
                        }
                    }
                }
            }

            let authorization = match req.headers().get("Authorization") {
                Some(a) => a,
                None => return Ok(AuthUser::unauthed()),
            };

            let header_value = authorization
                .to_str()
                .map_err(|_| ErrorResponse::new_status(401, "unauthorized"))?;
            let bearer = if header_value.starts_with("Bearer ") {
                &header_value[7..]
            } else {
                return Err(ErrorResponse::new_status(401, "unauthorized").into());
            };

            let data = web::Data::<AppState>::extract(&req).await.unwrap();

            let mut auth_service = new_auth_service(data.pool().clone());

            let authed_user = auth_service.authenticate_user(bearer).await;

            match authed_user {
                Ok(u) => Ok(AuthUser::from_user(u)),
                Err(e) => match e {
                    ServiceError::SpecificError(AuthError::TokenNotSet) => {
                        Err(ErrorResponse::new_status(401, "unauthorized"))
                    }
                    e => {
                        error!("Failed to authenticate user: {:?}", e);
                        Err(ErrorResponse::new_status(500, "internal server error"))
                    }
                },
            }
        })
    }
}

#[derive(Debug, Clone)]
struct ApubRequested {
    apub_requested: bool,
}

impl ApubRequested {
    pub fn from_req(req: &actix_web::HttpRequest) -> Self {
        let apub_requested = req
            .headers()
            .get("Accept")
            .map(|a| {
                let s = a.to_str().unwrap_or("");
                s.contains("application/activity+json")
                    || s.contains(
                        r#"application/ld+json; profile="https://www.w3.org/ns/activitystreams""#,
                    )
            })
            .unwrap_or(false);
        Self { apub_requested }
    }

    pub fn apub_requested(&self) -> bool {
        self.apub_requested
    }
}

impl FromRequest for ApubRequested {
    type Error = ErrorResponse;
    type Future = Pin<Box<dyn Future<Output = Result<ApubRequested, Self::Error>>>>;

    fn from_request(
        req: &actix_web::HttpRequest,
        _payload: &mut actix_web::dev::Payload,
    ) -> Self::Future {
        let req = req.clone();
        Box::pin(async move { Ok(ApubRequested::from_req(&req)) })
    }
}

fn new_id_getter_service(config: Config) -> IDGetterService {
    IDGetterService::new(config)
}

#[derive(Debug, Serialize)]
struct ErrorResponse {
    message: String,
    status: i32,
}

impl ErrorResponse {
    pub fn new_status(status: i32, message: impl Into<String>) -> Self {
        let msg = message.into();
        tracing::debug!("new error: {} {}", status, &msg);
        Self {
            message: msg.clone(),
            status,
        }
    }
}

impl<T> Into<Result<T, ErrorResponse>> for ErrorResponse {
    fn into(self) -> Result<T, ErrorResponse> {
        Err(self)
    }
}

impl<T: Debug> From<ServiceError<T>> for ErrorResponse {
    fn from(value: ServiceError<T>) -> Self {
        match value {
            ServiceError::SpecificError(e) => {
                error!("Specific error not handled: {:?}", &e);
                ErrorResponse::new_status(500, "internal server error")
            }
            ServiceError::MiscError(e) => {
                error!("Misc error: {:?}", &e);
                ErrorResponse::new_status(e.status_code(), e.message())
            }
        }
    }
}

impl Display for ErrorResponse {
    fn fmt(&self, f: &mut Formatter<'_>) -> std::fmt::Result {
        write!(f, "{}", self.message)
    }
}

impl actix_web::ResponseError for ErrorResponse {
    fn status_code(&self) -> actix_web::http::StatusCode {
        actix_web::http::StatusCode::from_u16(self.status as u16).unwrap()
    }
}

fn ise<T: Into<ErrorResponse> + Debug, S>(error: T) -> Result<S, ErrorResponse> {
    error!("Internal server error: {:?}", &error);
    Err(error.into())
}

#[derive(Debug, Deserialize)]
struct RegisterBody {
    pub username: String,
    pub nickname: String,
    pub password: String,
}

impl Into<UserCreateRequest> for RegisterBody {
    fn into(self) -> UserCreateRequest {
        UserCreateRequestBuilder::default()
            .username(self.username)
            .password(self.password)
            .nickname(self.nickname)
            .build()
            .unwrap()
    }
}

#[derive(Debug, Serialize)]
struct RegisterResponse {
    user_id: Simple,
}

#[post("/register")]
async fn register(
    body: web::Json<RegisterBody>,
    data: web::Data<AppState>,
) -> Result<impl Responder, ErrorResponse> {
    let mut us = new_user_service(data.pool().clone());
    let req = us.create_user(&body.0.into()).await;
    match req {
        Ok(req) => Ok(HttpResponse::Ok().json(RegisterResponse {
            user_id: *req.user_id(),
        })),
        Err(e) => match e {
            ServiceError::SpecificError(e) => match e {
                UserCreateError::UsernameConflict => {
                    Err(ErrorResponse::new_status(400, "username exists"))
                }
            },
            _ => ise(e),
        },
    }
}

#[derive(Debug, Deserialize)]
struct LoginBody {
    username: String,
    password: String,
}

impl Into<UserLoginRequest> for LoginBody {
    fn into(self) -> UserLoginRequest {
        UserLoginRequestBuilder::default()
            .username(self.username)
            .password(self.password)
            .build()
            .unwrap()
    }
}

#[derive(Debug, Serialize)]
struct LoginResponse {
    token: String,
}

#[post("/login")]
async fn login(
    body: web::Json<LoginBody>,
    data: web::Data<AppState>,
) -> Result<impl Responder, ErrorResponse> {
    let mut us = new_user_service(data.pool().clone());
    let req = us.login_user(&body.0.into()).await;
    match req {
        Ok(req) => Ok(HttpResponse::Ok().json(LoginResponse {
            token: req.user_token().to_string(),
        })),
        Err(e) => match e {
            ServiceError::SpecificError(e) => match e {
                UserLoginError::AuthFailed => Err(ErrorResponse::new_status(401, "auth failed")),
            },
            _ => ise(e),
        },
    }
}

#[derive(Debug, Deserialize)]
pub struct PostRequest {
    pub content: Option<String>,
    pub privacy: PostPrivacy,
    pub reply_to_id: Option<Uuid>,
    pub repost_of_id: Option<Uuid>,
}

#[derive(Debug, Serialize)]
pub struct PostCreateResponse {
    pub post_id: Simple,
}

#[post("/post")]
async fn post_post(
    body: web::Json<PostRequest>,
    data: web::Data<AppState>,
    auth: AuthUser,
) -> Result<impl Responder, ErrorResponse> {
    let user = auth.must_auth()?;

    let pool = data.pool().clone();
    let mut post_service =
        new_post_create_service(pool.clone(), data.queue().clone(), data.config().clone());

    let post = match (body.repost_of_id, body.reply_to_id) {
        (None, None) => PostCreateRequest::Normal(
            PostCreateRequestNormalBuilder::default()
                .poster(user.id.into())
                .content(
                    body.content
                        .clone()
                        .ok_or(ErrorResponse::new_status(400, "content is null"))?,
                )
                .privacy(body.privacy)
                .build()
                .unwrap(),
        ),
        (Some(repost_of_id), None) => {
            if let Some(content) = body.content.clone() {
                PostCreateRequest::Quote(
                    PostCreateRequestQuoteBuilder::default()
                        .poster(user.id.into())
                        .content(content)
                        .privacy(body.privacy)
                        .repost_of(repost_of_id.into())
                        .build()
                        .unwrap(),
                )
            } else {
                PostCreateRequest::Repost(
                    PostCreateRequestRepostBuilder::default()
                        .poster(user.id.into())
                        .privacy(body.privacy)
                        .repost_of(repost_of_id.into())
                        .build()
                        .unwrap(),
                )
            }
        }
        (None, Some(reply_to_id)) => PostCreateRequest::Reply(
            PostCreateRequestReplyBuilder::default()
                .poster(user.id.into())
                .content(
                    body.content
                        .clone()
                        .ok_or_else(|| ErrorResponse::new_status(400, "content is null"))?,
                )
                .privacy(body.privacy)
                .reply_to(reply_to_id.into())
                .build()
                .unwrap(),
        ),
        _ => {
            return Err(ErrorResponse::new_status(
                400,
                "repost_of_id and reply_to_id cannot be set at the same time",
            ))
        }
    };

    let req = post_service.create_post(&post).await.map_err(|e| {
        use PostCreateError::*;
        match &e {
            ServiceError::SpecificError(s) => match s {
                RepostOfNotFound => ErrorResponse::new_status(404, "repost_of_id not found"),
                ReplyToNotFound => ErrorResponse::new_status(404, "reply_to not found"),
                DisallowedPrivacyForRepost => {
                    ErrorResponse::new_status(400, "only public or unlisted repost can be made")
                }
                _ => e.into(),
            },
            _ => e.into(),
        }
    })?;

    Ok(HttpResponse::Ok().json(PostCreateResponse { post_id: req }))
}

#[derive(Debug, Deserialize)]
struct UserChooseParams {
    user_spec: UserSpecifier,
}

#[put("/user/{user_spec}/follow")]
async fn user_create_follow(
    path: web::Path<UserChooseParams>,
    data: web::Data<AppState>,
    auth: AuthUser,
) -> Result<impl Responder, ErrorResponse> {
    let user = auth.must_auth()?;

    let pool = data.pool().clone();
    let mut follow_service =
        new_follow_service(pool.clone(), data.queue().clone(), data.config().clone());

    follow_service
        .follow_user(&UserSpecifier::from_id(user.id), &path.user_spec)
        .await?;

    Ok(HttpResponse::Ok().finish())
}

#[delete("/user/{user_spec}/follow")]
async fn user_delete_follow(
    path: web::Path<UserChooseParams>,
    data: web::Data<AppState>,
    auth: AuthUser,
) -> Result<impl Responder, ErrorResponse> {
    let user = auth.must_auth()?;

    let pool = data.pool().clone();
    let mut follow_service =
        new_follow_service(pool.clone(), data.queue().clone(), data.config().clone());

    follow_service
        .unfollow_user(&UserSpecifier::from_id(user.id), &path.user_spec)
        .await?;

    Ok(HttpResponse::Ok().finish())
}

#[get("/nodeinfo/2.0")]
async fn node_info_2_0(app: web::Data<AppState>) -> impl Responder {
    let config = app.config();
    let node_info = gen_node_info("2.0", config);
    HttpResponse::Ok()
        .content_type("application/json")
        .body(node_info.to_string())
}

#[get("/nodeinfo/2.1")]
async fn node_info_2_1(app: web::Data<AppState>) -> impl Responder {
    let config = app.config();
    let node_info = gen_node_info("2.1", config);
    HttpResponse::Ok()
        .content_type("application/json")
        .body(node_info.to_string())
}

fn gen_node_info(node_info_version: &str, config: &Config) -> serde_json::Value {
    json!({
        "version": node_info_version,
        "software": {
            "name": "lightpub",
            "version": "0.1",
            "repository": "https://github.com/lightpub-dev/lightpub",
        },
        "protocol": [
            "activitypub",
        ],
        "lightpub_backend": {"inbound": [], "outbound": []},
        "openRegistrations": false,
        "usage": {
            // "users": {
                // "total": get_total_users(),
            // }
        },
        "metadata": {
            "nodeName": config.instance.name,
            "nodeDescription": config.instance.description,
        },
    })
}

#[get("/.well-known/nodeinfo")]
async fn well_known_node_info(app: web::Data<AppState>) -> impl Responder {
    let link_2_0 = format!("{}/nodeinfo/2.0", app.base_url());
    let link_2_1 = format!("{}/nodeinfo/2.1", app.base_url());
    let body = json!({
        "links": [
            {
                "rel": "http://nodeinfo.diaspora.software/ns/schema/2.1",
                "href": link_2_0,
            },
            {
                "rel": "http://nodeinfo.diaspora.software/ns/schema/2.0",
                "href": link_2_1,
            },
        ]
    });

    HttpResponse::Ok()
        .content_type("application/json")
        .body(body.to_string())
}

#[derive(Debug, Deserialize)]
struct WebfingerQuery {
    resource: String,
}

#[get("/.well-known/webfinger")]
async fn webfinger(
    query: web::Query<WebfingerQuery>,
    app: web::Data<AppState>,
) -> Result<impl Responder, ErrorResponse> {
    let resource = urlencoding::decode(&query.resource).expect("url decode");
    let parts: Vec<&str> = resource.split(":").collect();
    if parts.len() != 2 {
        return Ok(HttpResponse::BadRequest().body("Invalid resource"));
    }
    if parts[0] != "acct" {
        return Ok(HttpResponse::BadRequest().body("Invalid resource"));
    }

    let acct_id = parts[1];
    let user_spec = if !acct_id.contains("@") {
        // contains username only
        UserSpecifier::from_username(acct_id.to_string(), None)
    } else {
        let parts: Vec<&str> = acct_id.split("@").collect();
        if parts.len() != 2 {
            return Ok(HttpResponse::BadRequest().body("Invalid resource"));
        }
        if parts[1] != app.config().hostname {
            return Ok(HttpResponse::NotFound().body("user not found"));
        }
        UserSpecifier::from_username(parts[0].to_string(), None)
    };
    let mut user_finder = new_local_user_finder_service(app.pool().clone());
    let user = user_finder
        .find_user_by_specifier(&user_spec)
        .await
        .map_err(|e| match e {
            ServiceError::SpecificError(LocalUserFindError::NotLocalUser) => {
                ErrorResponse::new_status(404, "user not found")
            }
            _ => e.into(),
        })?;

    let base_url = app.base_url();

    Ok(HttpResponse::Ok().content_type("application/json").body(
        json!({
            "subject": acct_id,
            "links": [
                {
                    "rel": "self",
                    "type": "application/activity+json",
                    "href": format!("{}/user/{}", base_url, user.id.to_string())
                }
            ]
        })
        .to_string(),
    ))
}

#[get("/.well-known/host-meta")]
async fn host_meta(app: web::Data<AppState>) -> HandlerResponse<impl Responder> {
    let base_url = app.base_url();
    let xml = r#"
    <?xml version="1.0" encoding="UTF-8"?>
    <XRD xmlns="http://docs.oasis-open.org/ns/xri/xrd-1.0">
        <Link rel="lrdd" type="application/xrd+xml"
            template="{{base_url}}/.well-known/webfinger?resource={uri}" />
    </XRD>
    "#
    .replace("{{base_url}}", &base_url);
    Ok(HttpResponse::Ok()
        .content_type("application/xrd+xml")
        .body(xml))
}

async fn do_like_helper(
    app: web::Data<AppState>,
    like: LikeActivity,
    is_add: bool,
) -> HandlerResponse<impl Responder> {
    // assume that the actor is already authenticated
    let actor_id = like.actor;

    let note_id = like.object;
    let mut post_service = new_post_create_service(
        app.pool().clone(),
        app.queue().clone(),
        app.config().clone(),
    );

    let reaction_content = like.content;
    let reaction = match reaction_content {
        None => None,
        Some(r) => Some(Reaction::try_from(r).map_err(|e| {
            warn!("Failed to parse reaction: {:?}", e);
            ErrorResponse::new_status(400, "invalid reaction")
        })?),
    };

    let action = if is_add {
        PostInteractionAction::Add
    } else {
        PostInteractionAction::Remove
    };

    match reaction {
        None => {
            // favorite
            post_service
                .modify_favorite(
                    &UserSpecifier::from_url(actor_id),
                    &PostSpecifier::from_uri(note_id),
                    false,
                    false,
                    action,
                )
                .await
                .map_err(|e| {
                    warn!("Failed to like post: {:?}", e);
                    ErrorResponse::new_status(500, "internal server error")
                })?;
        }
        Some(r) => {
            // reaction
            post_service
                .modify_reaction(
                    &UserSpecifier::from_url(actor_id),
                    &PostSpecifier::from_uri(note_id),
                    &r,
                    false,
                    action,
                )
                .await
                .map_err(|e| {
                    warn!("Failed to reaction post: {:?}", e);
                    ErrorResponse::new_status(500, "internal server error")
                })?;
        }
    }

    Ok(HttpResponse::Ok().finish())
}

#[post("/user/{user_spec}/inbox")]
async fn user_inbox(
    params: web::Path<UserChooseParams>,
    app: web::Data<AppState>,
    auth: AuthUser,
    body: web::Json<serde_json::Value>,
) -> HandlerResponse<impl Responder> {
    debug!("user_inbox: {:?}", params);
    debug!("{:?}", body);

    let authed_user = auth.must_auth()?;
    let id_getter = new_id_getter_service(app.config().clone());
    let authed_user_uri = id_getter.get_user_id(authed_user);

    // deserialize into ActivityPub activity
    let activity = lightpub_model::apub::Activity::deserialize(&body.0).map_err(|e| {
        warn!("Failed to deserialize activity: {:?}", e);
        ErrorResponse::new_status(400, "invalid activity")
    })?;

    let authfail = || {
        Err(ErrorResponse::new_status(
            401,
            "actor does not match key owner",
        ))
    };

    debug!("parsed activity {:#?}", activity);

    use lightpub_model::apub::Activity::*;
    match activity {
        Accept(a) => {
            let actor_id = a.actor;
            if authed_user_uri != actor_id {
                return authfail();
            }
            let req_spec = match a.object {
                lightpub_model::apub::IdOrObject::Id(id) => todo!("fetch object by id: {}", id),
                lightpub_model::apub::IdOrObject::Object(obj) => match obj {
                    AcceptableActivity::Follow(obj) => {
                        let object_actor_id = obj.actor;
                        let object_object_id = obj.object.get_id();
                        if actor_id != object_object_id {
                            return Err(ErrorResponse::new_status(
                                400,
                                "actor and object id mismatch",
                            ));
                        }
                        FollowRequestSpecifier::ActorPair(
                            UserSpecifier::URL(object_actor_id),
                            UserSpecifier::URL(object_object_id.to_string()),
                        )
                    }
                },
            };

            let mut follow_service = new_follow_service(
                app.pool().clone(),
                app.queue().clone(),
                app.config().clone(),
            );
            info!("accepting follow request of {:#?}", req_spec);
            let result = follow_service.follow_request_accepted(&req_spec).await?;
            info!(
                "follow request accepted: FollowRequestID: {} -> {}",
                result.follower_id.simple(),
                result.followee_id.simple()
            );
        }
        Follow(follow) => {
            let follow_id = follow
                .id
                .ok_or_else(|| ErrorResponse::new_status(400, "follow id is not set"))?;
            let actor_id = follow.actor;
            let object_id = follow.object.get_id().to_string();

            if authed_user_uri != actor_id {
                return authfail();
            }

            debug!("accepting follow request of {} -> {}", actor_id, object_id);
            let mut follow_service = new_follow_service(
                app.pool().clone(),
                app.queue().clone(),
                app.config().clone(),
            );
            follow_service
                .incoming_follow_request(&IncomingFollowRequest::ActorPair(
                    follow_id,
                    UserSpecifier::URL(actor_id),
                    UserSpecifier::URL(object_id),
                ))
                .await?;
        }
        Create(create) => {
            let actor_id = create.actor;

            if authed_user_uri != actor_id {
                return authfail();
            }

            // get object
            let object = {
                match create.object {
                    IdOrObject::Id(id) => {
                        // request object using id
                        let mut reqester_service =
                            new_apub_reqwester_service(app.queue().clone(), app.config());
                        if let CreatableObject::Note(object_note) =
                            reqester_service.fetch_post(&id).await?
                        {
                            Some(object_note)
                        } else {
                            warn!("object not found");
                            return Err(ErrorResponse::new_status(404, "object not found"));
                        }
                    }
                    IdOrObject::Object(CreatableObject::Note(note)) => Some(note),
                    IdOrObject::Object(CreatableObject::Tombstone(_)) => {
                        warn!("tombstone object is not allowed");
                        return Err(ErrorResponse::new_status(400, "invalid activity"));
                    }
                }
            }
            .ok_or_else(|| {
                warn!("create object_id not found");
                ErrorResponse::new_status(400, "invalid activity")
            })?;

            let object_attributed_to = &object.attributed_to;
            if object_attributed_to != actor_id.as_str() {
                return Err(ErrorResponse::new_status(
                    400,
                    "object's attribute_to does not match actor",
                ));
            }

            let post_request = &object.try_into().unwrap();

            let mut post_service = new_post_create_service(
                app.pool().clone(),
                app.queue().clone(),
                app.config().clone(),
            );
            let result = post_service.create_post(post_request).await;
            match result {
                Ok(post_id) => {
                    info!("post created: {}", post_id);
                }
                Err(ServiceError::SpecificError(PostCreateError::AlreadyExists)) => {
                    info!("post already exists, skip");
                }
                Err(e) => {
                    return Err(e.into());
                }
            }
        }
        Announce(announce) => {
            let actor_id = announce.actor;

            if authed_user_uri != actor_id {
                return authfail();
            }

            let repost_id = announce.id;
            let object_id = announce.object.get_id();
            let published_at = announce.published.to_utc();

            let (to, cc) = {
                let mut to = announce.to;
                let mut cc = announce.cc;
                let bto = announce.bto.unwrap_or_default();
                let bcc = announce.bcc.unwrap_or_default();
                to.extend(bto);
                cc.extend(bcc);
                (to, cc)
            };
            let privacy = {
                if to.contains(&PUBLIC.to_string()) {
                    PostPrivacy::Public
                } else if cc.contains(&PUBLIC.to_string()) {
                    PostPrivacy::Unlisted
                } else {
                    warn!("rejected: Announce object's privacy must be public or unlisted.");
                    return Err(ErrorResponse::new_status(400, "invalid activity"));
                }
            };

            let repost = PostCreateRequest::Repost(
                PostCreateRequestRepostBuilder::default()
                    .poster(UserSpecifier::from_url(actor_id))
                    .uri(repost_id)
                    .privacy(privacy)
                    .created_at(published_at)
                    .repost_of(PostSpecifier::from_uri(object_id))
                    .build()
                    .unwrap(),
            );

            let mut post_service = new_post_create_service(
                app.pool().clone(),
                app.queue().clone(),
                app.config().clone(),
            );
            let result = post_service.create_post(&repost).await;
            match result {
                Ok(post_id) => {
                    info!("repost created: {}", post_id);
                }
                Err(e) => match e {
                    ServiceError::SpecificError(PostCreateError::AlreadyExists) => {
                        info!("repost already exists, skip");
                    }
                    _ => {
                        return Err(e.into());
                    }
                },
            }
        }
        Reject(reject) => {
            let actor_id = reject.actor;

            if authed_user_uri != actor_id {
                return authfail();
            }

            match reject.object {
                RejectableActivity::Follow(f) => {
                    let follower_id = f.actor;
                    let followee_id = f.object.get_id();

                    if followee_id != actor_id {
                        return authfail();
                    }
                    let mut follow_service = new_follow_service(
                        app.pool().clone(),
                        app.queue().clone(),
                        app.config().clone(),
                    );
                    follow_service
                        .unfollow_user(
                            &UserSpecifier::from_url(follower_id),
                            &UserSpecifier::from_url(followee_id),
                        )
                        .await?;
                }
            }
        }
        Undo(undo) => {
            let actor_id = undo.actor;

            if authed_user_uri != actor_id {
                return authfail();
            }

            match undo.object {
                UndoableActivity::Follow(f) => {
                    let follower_id = f.actor;
                    let followee_id = f.object.get_id();

                    if follower_id != actor_id {
                        return authfail();
                    }
                    let mut follow_service = new_follow_service(
                        app.pool().clone(),
                        app.queue().clone(),
                        app.config().clone(),
                    );
                    follow_service
                        .unfollow_user(
                            &UserSpecifier::from_url(follower_id),
                            &UserSpecifier::from_url(followee_id),
                        )
                        .await?;
                }
                UndoableActivity::Like(like) => {
                    let actor_id = &like.actor;

                    if actor_id != authed_user_uri.as_str() {
                        return authfail();
                    }

                    do_like_helper(app, like, false).await?;
                }
            }
        }
        Delete(del) => {
            let actor_id = del.actor;

            if authed_user_uri != actor_id {
                return authfail();
            }

            let note_id = del.object.get_id();
            let mut post_service = new_post_create_service(
                app.pool().clone(),
                app.queue().clone(),
                app.config().clone(),
            );
            post_service
                .delete_post(
                    &PostSpecifier::from_uri(note_id),
                    &Some(authed_user.as_specifier()),
                )
                .await
                .map_err(|e| {
                    warn!("Failed to delete post: {:?}", e);
                    ErrorResponse::new_status(500, "internal server error")
                })?;
        }
        Like(like) => {
            let actor_id = &like.actor;

            if authed_user_uri.as_str() != actor_id {
                return authfail();
            }

            do_like_helper(app, like, true).await?;
        }
    }

    Ok(HttpResponse::Ok().finish())
}

#[get("/user/{user_spec}")]
async fn user_get(
    params: web::Path<UserChooseParams>,
    app: web::Data<AppState>,
) -> HandlerResponse<impl Responder> {
    let renderer_service = new_apub_renderer_service(app.config().clone());
    let user_spec = &params.user_spec;
    let mut user_finder = new_local_user_finder_service(app.pool().clone());
    let user = user_finder
        .find_user_by_specifier(user_spec)
        .await
        .map_err(|e| match e {
            ServiceError::SpecificError(LocalUserFindError::NotLocalUser) => {
                ErrorResponse::new_status(404, "user not found")
            }
            _ => e.into(),
        })?;

    let user = renderer_service.render_user(&user).map_err(|e| {
        error!("Failed to render user: {:?}", e);
        ErrorResponse::new_status(500, "internal server error")
    })?;
    let actor = Actor::Person(user).with_context();
    let user_json = serde_json::to_string(&actor).unwrap();
    debug!("user_json: {}", user_json);

    Ok(HttpResponse::Ok()
        .content_type("application/activity+json")
        .body(user_json))
}

#[derive(MultipartForm)]
struct UploadRequest {
    file: actix_multipart::form::tempfile::TempFile,
}

#[post("/upload")]
async fn file_upload(
    app: web::Data<AppState>,
    auth: AuthUser,
    file: MultipartForm<UploadRequest>,
) -> HandlerResponse<impl Responder> {
    let authed_user = auth.must_auth()?;

    let upload_dir = &app.config().upload_dir;
    let filename = generate_uuid();
    let fileext = file
        .file
        .file_name
        .clone()
        .map(|s| {
            s.rsplit(".")
                .next()
                .map(|s| {
                    if !s.is_ascii() {
                        "".into()
                    } else {
                        format!(".{}", s)
                    }
                })
                .unwrap_or("".into())
        })
        .unwrap_or("".into());
    if fileext == "" {
        return Err(ErrorResponse::new_status(400, "filename has no extension"));
    }
    let filepath = format!("{}/{}{}", upload_dir, filename, fileext);

    let file = file.into_inner();
    match file.file.file.persist_noclobber(filepath) {
        Err(e) => {
            error!("Failed to save file: {:?}", e);
            return Err(ErrorResponse::new_status(500, "internal server error"));
        }
        Ok(_) => {
            let mut upload_service =
                new_db_file_upload_service(app.pool().clone(), app.config().clone());
            let result = upload_service
                .upload_file(&authed_user.id.into(), filename, &fileext)
                .await;
            match result {
                Ok(_) => Ok(HttpResponse::Ok().finish()),
                Err(e) => {
                    error!("Failed to save file: {:?}", e);
                    Err(ErrorResponse::new_status(500, "internal server error"))
                }
            }
        }
    }
}

#[derive(Debug, Deserialize)]
struct UpdateMyProfileRequest {
    nickname: String,
    bio: String,
    avatar_id: Option<String>,
}

#[put("/user")]
async fn update_my_profile(
    app: web::Data<AppState>,
    body: web::Json<UpdateMyProfileRequest>,
    auth: AuthUser,
) -> HandlerResponse<impl Responder> {
    let authed_user = auth.must_auth()?;

    let mut profile_service = new_db_user_profile_service(app.pool().clone(), app.config().clone());

    let avatar_id = {
        if let Some(avatar_id) = &body.avatar_id {
            let u = avatar_id
                .parse::<uuid::Uuid>()
                .map_err(|_| ErrorResponse::new_status(400, "avatar_id is not valid"))?;
            Some(u.simple())
        } else {
            None
        }
    };

    let update = UserProfileUpdate::new(body.nickname.clone(), body.bio.clone(), avatar_id);

    profile_service
        .update_user_profile(&authed_user.id.into(), &update)
        .await?;

    Ok(HttpResponse::Ok().finish())
}

#[derive(Debug, Deserialize, Serialize, Clone)]
struct OutboxQuery {
    before_date: Option<chrono::DateTime<chrono::Utc>>,
    #[serde(default)]
    page: bool,
}

#[get("/user/{user_spec}/outbox")]
async fn get_user_outbox(
    app: web::Data<AppState>,
    path: web::Path<UserChooseParams>,
    auth: AuthUser,
    query: web::Query<OutboxQuery>,
) -> HandlerResponse<impl Responder> {
    if !query.page {
        return Ok(HttpResponse::Ok().json(CollectionResponse::from_first(
            CollectionType::OrderedCollection,
            {
                let base_url = app.config().base_url();
                let new_query = OutboxQuery {
                    before_date: None,
                    page: true,
                };
                format!(
                    "{}/user/{}/outbox?{}",
                    base_url,
                    path.user_spec,
                    serde_urlencoded::to_string(new_query).unwrap()
                )
            },
            None,
        )));
    }

    let user_spec = &path.user_spec;
    let viewer = &auth.may_auth()?.as_ref().map(|u| u.id.into());

    let mut post_service = new_db_user_post_service(
        app.pool().clone(),
        app.queue().clone(),
        app.config().clone(),
    );

    let limit = 20;
    let options = FetchUserPostsOptions::new(limit + 1, query.before_date, true);
    let posts: Vec<_> = post_service
        .fetch_user_posts(user_spec, viewer, &options)
        .await
        .map_err(|e| {
            error!("Failed to fetch user posts: {:?}", e);
            ErrorResponse::new_status(500, "internal server error")
        })?
        .into_iter()
        .filter(|p| {
            p.content().is_some() // filter out reposts
        })
        .collect();

    let renderer_service = new_apub_renderer_service(app.config().clone());
    let mut rendered_posts = Vec::with_capacity(posts.len());
    for p in posts {
        let rendered = renderer_service.render_post(&p).map_err(|e| {
            error!("Failed to render post: {:?}", e);
            ErrorResponse::new_status(500, "internal server error")
        })?;
        rendered_posts.push(PaginatableWrapper::new(
            match rendered.note().clone() {
                RenderedNoteObject::Create(c) => c,
                RenderedNoteObject::Announce(_) => {
                    // We can assume that reposts are not in `posts`
                    unreachable!("Announce object should not be in outbox")
                }
            },
            p.created_at().clone(),
        ));
    }

    let paginated =
        PaginatedResponse::from_result(rendered_posts, limit.try_into().unwrap(), |last| {
            let base_url = app.config().base_url();
            let mut new_query = (*query).clone();
            new_query.before_date = Some(*last);

            format!(
                "{}/user/{}/outbox?{}",
                base_url,
                user_spec,
                serde_urlencoded::to_string(new_query).unwrap()
            )
        });
    let paginated = CollectionPageResponse::from_paginated_response(
        CollectionPageType::OrderedCollectionPage,
        paginated,
        format!("{}/user/{}/outbox", app.config().base_url(), path.user_spec),
        None,
    );
    Ok(HttpResponse::Ok().json(paginated.with_context()))
}

#[derive(Debug, Deserialize, Serialize, Clone)]
struct GetUserPostsQuery {
    limit: Option<i64>,
    before_date: Option<chrono::DateTime<chrono::Utc>>,
    #[serde(default)]
    page: bool,
}

#[derive(Debug, Deserialize, Serialize, Clone)]
struct PostChooseParams {
    post_id: Uuid,
}

#[get("/post/{post_id}")]
async fn get_single_post(
    app: web::Data<AppState>,
    path: web::Path<PostChooseParams>,
    auth: AuthUser,
) -> HandlerResponse<impl Responder> {
    let viewer = auth.may_auth()?.as_ref().map(|u| u.id.into());

    let post_id = &PostSpecifier::from_id(path.post_id);
    let mut post_service = new_db_user_post_service(
        app.pool().clone(),
        app.queue().clone(),
        app.config().clone(),
    );

    let post = post_service.fetch_single_post(post_id, &viewer).await;

    match post {
        Err(e) => {
            if let Some(err) = e.downcast_ref::<PostFetchError>() {
                match err {
                    PostFetchError::PostNotFound => {
                        return Err(ErrorResponse::new_status(404, "post not found"))
                    }
                }
            }
            error!("Failed to fetch post: {:?}", e);
            Err(ErrorResponse::new_status(500, "internal server error"))
        }
        Ok(p) => Ok(HttpResponse::Ok().json(p)),
    }
}

#[delete("/post/{post_id}")]
async fn delete_single_post(
    app: web::Data<AppState>,
    path: web::Path<PostChooseParams>,
    auth: AuthUser,
) -> HandlerResponse<impl Responder> {
    let viewer = auth.must_auth()?;

    let post_id = &PostSpecifier::from_id(path.post_id);
    let mut post_service = new_post_create_service(
        app.pool().clone(),
        app.queue().clone(),
        app.config().clone(),
    );

    let post = post_service
        .delete_post(post_id, &Some(viewer.as_specifier()))
        .await;

    match post {
        Err(e) => match e.downcast_ref::<PostDeleteError>() {
            Some(PostDeleteError::PostNotFound) => {
                return Err(ErrorResponse::new_status(404, "post not found"));
            }
            Some(PostDeleteError::Unauthorized) => {
                return Err(ErrorResponse::new_status(
                    403,
                    "only the author can delete the post",
                ));
            }
            None => {
                error!("Failed to delete post: {:?}", e);
                return Err(ErrorResponse::new_status(500, "internal server error"));
            }
        },
        Ok(_) => Ok(HttpResponse::Ok().finish()),
    }
}

#[get("/user/{user_spec}/posts")]
async fn get_user_posts(
    app: web::Data<AppState>,
    path: web::Path<UserChooseParams>,
    auth: AuthUser,
    query: web::Query<GetUserPostsQuery>,
) -> HandlerResponse<impl Responder> {
    let user_spec = &path.user_spec;
    let viewer = &auth.may_auth()?.as_ref().map(|u| u.id.into());

    let mut post_service = new_db_user_post_service(
        app.pool().clone(),
        app.queue().clone(),
        app.config().clone(),
    );

    let limit = query
        .limit
        .map(|li| if 0 < li && li <= 100 { li } else { 20 })
        .unwrap_or(20);
    let options = FetchUserPostsOptions::new(limit + 1, query.before_date, false);
    let posts = post_service
        .fetch_user_posts(user_spec, viewer, &options)
        .await
        .map_err(|e| {
            error!("Failed to fetch user posts: {:?}", e);
            ErrorResponse::new_status(500, "internal server error")
        })?;
    let paginated = PaginatedResponse::from_result(posts, limit.try_into().unwrap(), |last| {
        let base_url = app.config().base_url();
        let mut new_query = (*query).clone();
        new_query.before_date = Some(*last);

        format!(
            "{}/user/{}/posts?{}",
            base_url,
            user_spec,
            serde_urlencoded::to_string(new_query).unwrap()
        )
    });
    Ok(HttpResponse::Ok().json(paginated))
}

#[derive(Debug, Clone, Deserialize, Serialize)]
struct ListFollowOptions {
    pub limit: Option<i64>,
    pub before_date: Option<chrono::DateTime<chrono::Utc>>,
    #[serde(default)]
    pub page: bool,
}

#[get("/user/{user_spec}/followers")]
async fn get_user_followers(
    app: web::Data<AppState>,
    path: web::Path<UserChooseParams>,
    query: web::Query<ListFollowOptions>,
    apub: ApubRequested, // auth: AuthUser,
) -> HandlerResponse<impl Responder> {
    if apub.apub_requested() && !query.page {
        return Ok(HttpResponse::Ok().json(CollectionResponse::from_first(
            CollectionType::OrderedCollection,
            {
                let base_url = app.config().base_url();
                let mut new_query = (*query).clone();
                new_query.page = true;
                format!(
                    "{}/user/{}/followers?{}",
                    base_url,
                    path.user_spec,
                    serde_urlencoded::to_string(new_query).unwrap()
                )
            },
            None,
        )));
    }

    let user_spec = &path.user_spec;
    // let viewer = &auth.may_auth()?.as_ref().map(|u| u.id.into());

    let mut follow_service = new_follow_service(
        app.pool().clone(),
        app.queue().clone(),
        app.config().clone(),
    );

    let limit = query
        .limit
        .map(|li| if 0 < li && li <= 100 { li } else { 20 })
        .unwrap_or(20);
    let options = FetchFollowListOptions::new(limit, query.before_date);
    let followers = follow_service
        .fetch_follower_list(user_spec, &options)
        .await
        .map_err(|e| {
            error!("Failed to fetch user followers: {:?}", e);
            ErrorResponse::new_status(500, "internal server error")
        })?;

    if !apub.apub_requested() {
        let paginated =
            PaginatedResponse::from_result(followers, limit.try_into().unwrap(), |last| {
                let base_url = app.config().base_url();
                let mut new_query = (*query).clone();
                new_query.before_date = Some(*last);

                format!(
                    "{}/user/{}/followers?{}",
                    base_url,
                    user_spec,
                    serde_urlencoded::to_string(new_query).unwrap()
                )
            });

        Ok(HttpResponse::Ok().json(paginated))
    } else {
        let id_service = new_id_getter_service(app.config().clone());
        let items = followers
            .into_iter()
            .map(|f| PaginatableWrapper::new(id_service.get_user_id(&f), f.created_at().clone()))
            .collect();

        let base_url = app.config().base_url();
        let paginated = CollectionPageResponse::from_paginated_response(
            CollectionPageType::OrderedCollectionPage,
            PaginatedResponse::from_result(items, limit.try_into().unwrap(), |last| {
                let base_url = app.config().base_url();
                let mut new_query = (*query).clone();
                new_query.before_date = Some(*last);

                format!(
                    "{}/user/{}/followers?{}",
                    base_url,
                    user_spec,
                    serde_urlencoded::to_string(new_query).unwrap()
                )
            }),
            format!("{}/user/{}/followers", base_url, path.user_spec,),
            None,
        );

        Ok(HttpResponse::Ok().json(paginated))
    }
}

#[get("/user/{user_spec}/following")]
async fn get_user_following(
    app: web::Data<AppState>,
    path: web::Path<UserChooseParams>,
    query: web::Query<ListFollowOptions>,
    apub: ApubRequested,
    // auth: AuthUser,
) -> HandlerResponse<impl Responder> {
    if apub.apub_requested() && !query.page {
        return Ok(HttpResponse::Ok().json(CollectionResponse::from_first(
            CollectionType::OrderedCollection,
            {
                let base_url = app.config().base_url();
                let mut new_query = (*query).clone();
                new_query.page = true;
                format!(
                    "{}/user/{}/following?{}",
                    base_url,
                    path.user_spec,
                    serde_urlencoded::to_string(new_query).unwrap()
                )
            },
            None,
        )));
    }

    let user_spec = &path.user_spec;
    // let viewer = &auth.may_auth()?.as_ref().map(|u| u.id.into());

    let mut follow_service = new_follow_service(
        app.pool().clone(),
        app.queue().clone(),
        app.config().clone(),
    );

    let limit = query
        .limit
        .map(|li| if 0 < li && li <= 100 { li } else { 20 })
        .unwrap_or(20);
    let options = FetchFollowListOptions::new(limit, query.before_date);
    let followers = follow_service
        .fetch_following_list(user_spec, &options)
        .await
        .map_err(|e| {
            error!("Failed to fetch user following: {:?}", e);
            ErrorResponse::new_status(500, "internal server error")
        })?;

    if !apub.apub_requested() {
        let paginated =
            PaginatedResponse::from_result(followers, limit.try_into().unwrap(), |last| {
                let base_url = app.config().base_url();
                let mut new_query = (*query).clone();
                new_query.before_date = Some(*last);

                format!(
                    "{}/user/{}/following?{}",
                    base_url,
                    user_spec,
                    serde_urlencoded::to_string(new_query).unwrap()
                )
            });

        Ok(HttpResponse::Ok().json(paginated))
    } else {
        let id_service = new_id_getter_service(app.config().clone());
        let items = followers
            .into_iter()
            .map(|f| PaginatableWrapper::new(id_service.get_user_id(&f), f.created_at().clone()))
            .collect();

        let base_url = app.config().base_url();
        let paginated = CollectionPageResponse::from_paginated_response(
            CollectionPageType::OrderedCollectionPage,
            PaginatedResponse::from_result(items, limit.try_into().unwrap(), |last| {
                let base_url = app.config().base_url();
                let mut new_query = (*query).clone();
                new_query.before_date = Some(*last);

                format!(
                    "{}/user/{}/following?{}",
                    base_url,
                    user_spec,
                    serde_urlencoded::to_string(new_query).unwrap()
                )
            }),
            format!("{}/user/{}/following", base_url, path.user_spec,),
            None,
        );

        Ok(HttpResponse::Ok().json(paginated))
    }
}

#[derive(Debug, Deserialize, Clone, Serialize)]
struct TimelineQuery {
    pub limit: Option<i64>,
    #[serde(default)]
    pub public: bool,
    pub before_date: Option<chrono::DateTime<chrono::Utc>>,
}

#[get("/timeline")]
async fn timeline(
    app: web::Data<AppState>,
    auth: AuthUser,
    query: web::Query<TimelineQuery>,
) -> HandlerResponse<impl Responder> {
    let authed_user = auth.must_auth()?;

    let mut post_service = new_db_user_post_service(
        app.pool().clone(),
        app.queue().clone(),
        app.config().clone(),
    );

    let limit = query
        .limit
        .map(|li| if 0 < li && li <= 100 { li } else { 20 })
        .unwrap_or(20);
    let options = TimelineOptions::new(limit + 1, query.before_date, query.public);
    let posts = post_service
        .fetch_timeline(&authed_user.id.into(), &options)
        .await
        .map_err(|e| {
            error!("Failed to fetch user timeline: {:?}", e);
            ErrorResponse::new_status(500, "internal server error")
        })?;
    let paginated = PaginatedResponse::from_result(posts, limit.try_into().unwrap(), |last| {
        let base_url = app.config().base_url();
        let mut new_query = (*query).clone();
        new_query.before_date = Some(*last);

        format!(
            "{}/timeline?{}",
            base_url,
            serde_urlencoded::to_string(new_query).unwrap()
        )
    });
    Ok(HttpResponse::Ok().json(paginated))
}

#[put("/post/{post_id}/favorite")]
async fn add_post_favorite(
    app: web::Data<AppState>,
    path: web::Path<PostChooseParams>,
    auth: AuthUser,
) -> HandlerResponse<impl Responder> {
    let authed_user = auth.must_auth()?;

    let post_spec = &path.post_id.into();
    let mut post_service = new_post_create_service(
        app.pool().clone(),
        app.queue().clone(),
        app.config().clone(),
    );

    post_service
        .modify_favorite(
            &authed_user.id.into(),
            post_spec,
            false,
            false,
            PostInteractionAction::Add,
        )
        .await
        .map_err(|e| {
            error!("Failed to add favorite: {:?}", e);
            ErrorResponse::new_status(500, "internal server error")
        })?;

    Ok(HttpResponse::Ok().finish())
}

#[put("/post/{post_id}/bookmark")]
async fn add_post_bookmark(
    app: web::Data<AppState>,
    path: web::Path<PostChooseParams>,
    auth: AuthUser,
) -> HandlerResponse<impl Responder> {
    let authed_user = auth.must_auth()?;

    let post_spec = &path.post_id.into();
    let mut post_service = new_post_create_service(
        app.pool().clone(),
        app.queue().clone(),
        app.config().clone(),
    );

    post_service
        .modify_favorite(
            &authed_user.id.into(),
            post_spec,
            false,
            true,
            PostInteractionAction::Add,
        )
        .await
        .map_err(|e| {
            error!("Failed to add favorite: {:?}", e);
            ErrorResponse::new_status(500, "internal server error")
        })?;

    Ok(HttpResponse::Ok().finish())
}

#[delete("/post/{post_id}/favorite")]
async fn delete_post_favorite(
    app: web::Data<AppState>,
    path: web::Path<PostChooseParams>,
    auth: AuthUser,
) -> HandlerResponse<impl Responder> {
    let authed_user = auth.must_auth()?;

    let post_spec = &path.post_id.into();
    let mut post_service = new_post_create_service(
        app.pool().clone(),
        app.queue().clone(),
        app.config().clone(),
    );

    post_service
        .modify_favorite(
            &authed_user.id.into(),
            post_spec,
            false,
            false,
            PostInteractionAction::Remove,
        )
        .await
        .map_err(|e| {
            error!("Failed to remove favorite: {:?}", e);
            ErrorResponse::new_status(500, "internal server error")
        })?;

    Ok(HttpResponse::Ok().finish())
}

#[delete("/post/{post_id}/bookmark")]
async fn delete_post_bookmark(
    app: web::Data<AppState>,
    path: web::Path<PostChooseParams>,
    auth: AuthUser,
) -> HandlerResponse<impl Responder> {
    let authed_user = auth.must_auth()?;

    let post_spec = &path.post_id.into();
    let mut post_service = new_post_create_service(
        app.pool().clone(),
        app.queue().clone(),
        app.config().clone(),
    );

    post_service
        .modify_favorite(
            &authed_user.id.into(),
            post_spec,
            false,
            true,
            PostInteractionAction::Remove,
        )
        .await
        .map_err(|e| {
            error!("Failed to remove favorite: {:?}", e);
            ErrorResponse::new_status(500, "internal server error")
        })?;

    Ok(HttpResponse::Ok().finish())
}

#[derive(Debug, Serialize, Deserialize)]
struct PostReactionRequest {
    reaction: String,
    add: bool,
}

#[post("/post/{post_id}/reaction")]
async fn modify_post_reaction(
    app: web::Data<AppState>,
    path: web::Path<PostChooseParams>,
    body: web::Json<PostReactionRequest>,
    auth: AuthUser,
) -> HandlerResponse<impl Responder> {
    let authed_user = auth.must_auth()?;

    let post_spec = &path.post_id.into();
    let mut post_service = new_post_create_service(
        app.pool().clone(),
        app.queue().clone(),
        app.config().clone(),
    );

    let reaction = Reaction::try_from(body.reaction.clone());
    let reaction = match reaction {
        Ok(r) => r,
        Err(e) => match e {
            ReactionError::InvalidUnicodeEmoji => {
                return Err(ErrorResponse::new_status(400, "invalid reaction"));
            }
        },
    };

    post_service
        .modify_reaction(
            &authed_user.id.into(),
            post_spec,
            &reaction,
            false,
            if body.add {
                PostInteractionAction::Add
            } else {
                PostInteractionAction::Remove
            },
        )
        .await
        .map_err(|e| {
            error!("Failed to add favorite: {:?}", e);
            ErrorResponse::new_status(500, "internal server error")
        })?;

    Ok(HttpResponse::Ok().finish())
}

#[derive(Parser)]
#[command(version, about, long_about = None)]
struct Cli {
    /// Sets a custom config file
    #[arg(short, long, value_name = "FILE")]
    config: Option<PathBuf>,
}

#[actix_web::main]
async fn main() -> std::io::Result<()> {
    tracing_subscriber::fmt()
        .with_max_level(tracing::Level::DEBUG)
        .with_test_writer()
        .init();

    let cli = Cli::parse();
    let config = cli.config.unwrap_or("lightpub.yml.sample".into());

    let mut file = std::fs::File::open(config).unwrap();
    let mut contents = String::new();
    file.read_to_string(&mut contents)
        .expect("Unable to read file");
    let config: Config = serde_yaml::from_str(&contents).expect("Unable to deserialize YAML");

    // connect to db
    let conn_str = format!(
        "mysql://{}:{}@{}:{}/{}",
        config.database.user,
        config.database.password,
        config.database.host,
        config.database.port,
        config.database.name
    );
    let pool = MySqlPoolOptions::new()
        .max_connections(config.database.max_connections)
        .idle_timeout(Duration::from_secs(30))
        .acquire_timeout(Duration::from_secs(5))
        .connect(&conn_str)
        .await
        .expect("connect to database");
    tracing::info!("Connected to database");

    // connect to queue
    let queue_uri = format!(
        "amqp://{}:{}@{}:{}",
        config.queue.user, config.queue.password, config.queue.host, config.queue.port
    );
    let queue = lapin::Connection::connect(&queue_uri, ConnectionProperties::default())
        .await
        .expect("connect to amqp queue");
    let queue_builder = QueuedApubRequester::prepare(&queue)
        .await
        .expect("initialize amqp queue");
    tracing::info!("Connected to queue");

    // create upload_dir
    let upload_dir = config.upload_dir.clone();
    web::block(move || {
        std::fs::create_dir_all(upload_dir).expect("failed to create upload_dir");
    })
    .await
    .unwrap();

    let app_state = AppState::new(pool, queue_builder, config.clone());

    HttpServer::new(move || {
        let cors = Cors::default()
            .allowed_origin("http://localhost:5173")
            .allowed_methods(vec!["GET", "POST", "PUT", "DELETE", "PATCH"])
            .allowed_headers(vec![
                header::AUTHORIZATION,
                header::CONTENT_TYPE,
                header::ACCEPT,
            ]);

        App::new()
            .app_data(web::Data::new(app_state.clone()))
            .wrap(cors)
            .wrap(Logger::default())
            .service(register)
            .service(login)
            .service(post_post)
            .service(user_create_follow)
            .service(user_delete_follow)
            .service(webfinger)
            .service(node_info_2_0)
            .service(node_info_2_1)
            .service(host_meta)
            .service(well_known_node_info)
            .service(user_inbox)
            .service(user_get)
            .service(file_upload)
            .service(update_my_profile)
            .service(get_user_posts)
            .service(get_user_followers)
            .service(get_user_following)
            .service(get_user_outbox)
            .service(timeline)
            .service(get_single_post)
            .service(delete_single_post)
            .service(add_post_favorite)
            .service(add_post_bookmark)
            .service(delete_post_favorite)
            .service(delete_post_bookmark)
            .service(modify_post_reaction)
    })
    .bind(("0.0.0.0", 8000))?
    .run()
    .await
}
