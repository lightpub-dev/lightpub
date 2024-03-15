mod config;
pub mod models;
pub mod services;
pub mod state;
pub mod utils;

use crate::models::apub::context::ContextAttachable;
use crate::models::apub::{Actor, CreatableObject, HasId, IdOrObject, PUBLIC};

use crate::utils::key::VerifyError;
use crate::{
    services::{
        apub::{new_apub_renderer_service, new_apub_reqwester_service},
        db::{new_follow_service, new_post_create_service},
        FollowRequestSpecifier, IncomingFollowRequest, PostCreateError, PostCreateRequest,
        PostCreateRequestNormalBuilder, PostCreateRequestQuoteBuilder,
        PostCreateRequestReplyBuilder, PostCreateRequestRepostBuilder,
    },
    utils::post::PostSpecifier,
};
use actix_web::{
    delete, get, middleware::Logger, post, put, web, App, FromRequest, HttpResponse, HttpServer,
    Responder,
};
use config::Config;
use models::http::{HeaderMapWrapper, Method};
use models::{PostPrivacy, User};
use serde::{Deserialize, Serialize};
use serde_json::json;
use services::db::{new_all_user_finder_service, new_db_key_fetcher_service};
use services::{
    db::{new_auth_service, new_local_user_finder_service},
    id::IDGetterService,
    AuthError, LocalUserFindError, ServiceError, UserCreateRequest, UserCreateRequestBuilder,
    UserLoginError, UserLoginRequest, UserLoginRequestBuilder,
};
use sqlx::mysql::MySqlPoolOptions;
use state::AppState;
use std::borrow::BorrowMut;
use std::{
    fmt::{Debug, Display, Formatter},
    future::Future,
    io::Read,
    pin::Pin,
};
use tracing::{debug, error, info, warn};
use utils::key::{verify_signature, KeyFetcher};
use utils::user::UserSpecifier;
use uuid::{fmt::Simple, Uuid};

use crate::services::db::new_user_service;
use utoipa::OpenApi;
use utoipa::ToSchema;
use utoipa_swagger_ui::SwaggerUi;

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

    pub fn must_auth(&self) -> Result<&User, ErrorResponse> {
        match &self.authed_user {
            Some(u) => Ok(&u),
            None => Err(ErrorResponse::new_status(401, "unauthorized")),
        }
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
            let mut key_fetcher =
                new_db_key_fetcher_service(data.pool().clone(), data.config().clone());
            let sig_result = verify_signature(
                HeaderMapWrapper::from_actix(req.headers()),
                Method::from_actix(req.method()),
                req.path(),
                key_fetcher.borrow_mut() as &mut (dyn KeyFetcher + Send + Sync),
            )
            .await;
            match sig_result {
                Ok(u) => {
                    let mut user_finder =
                        new_all_user_finder_service(data.pool().clone(), data.config().clone());
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

            let authorization = req
                .headers()
                .get("Authorization")
                .ok_or(ErrorResponse::new_status(401, "unauthorized"))?;
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
                        todo!()
                    }
                    _ => todo!(),
                },
            }
        })
    }
}

#[get("/api/")]
async fn hello() -> impl Responder {
    HttpResponse::Ok().body("Hello world!")
}

#[post("/echo")]
async fn echo(req_body: String) -> impl Responder {
    HttpResponse::Ok().body(req_body)
}

fn new_id_getter_service(config: Config) -> IDGetterService {
    IDGetterService::new(config)
}

#[derive(ToSchema, Debug, Serialize)]
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

#[derive(ToSchema, Debug, Deserialize)]
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

#[derive(ToSchema, Debug, Serialize)]
struct RegisterResponse {
    user_id: Simple,
}

#[utoipa::path(
    post,
    request_body = RegisterBody,
    responses(
        (status = 200, description = "Registered User", body = RegisterResponse),
    ),
)]
#[post("/register")]
async fn register(
    body: web::Json<RegisterBody>,
    data: web::Data<AppState>,
) -> Result<impl Responder, ErrorResponse> {
    let mut us = new_user_service(data.pool().clone());
    let req = us.create_user(&body.0.into()).await.unwrap();
    Ok(HttpResponse::Ok().json(RegisterResponse {
        user_id: *req.user_id(),
    }))
}

#[derive(ToSchema, Debug, Deserialize)]
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

#[derive(ToSchema, Debug, Serialize)]
struct LoginResponse {
    token: String,
}

#[utoipa::path(
    post,
    request_body = LoginBody,
    responses(
        (status = 200, description = "Logged in", body = LoginResponse),
        (status = 401, description = "Auth failed")
    ),
)]
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

#[derive(Debug, Deserialize, ToSchema)]
pub struct PostRequest {
    pub content: Option<String>,
    pub privacy: PostPrivacy,
    pub reply_to_id: Option<Uuid>,
    pub repost_of_id: Option<Uuid>,
}

#[derive(Debug, Serialize, ToSchema)]
pub struct PostCreateResponse {
    pub post_id: Simple,
}

#[utoipa::path(
    post,
    request_body = PostRequest,
    responses(
        (status = 200, description = "Created post", body = PostCreateResponse),
    ),
)]
#[post("/post")]
async fn post_post(
    body: web::Json<PostRequest>,
    data: web::Data<AppState>,
    auth: AuthUser,
) -> Result<impl Responder, ErrorResponse> {
    let user = auth.must_auth()?;

    let pool = data.pool().clone();
    let mut post_service = new_post_create_service(pool.clone(), data.config().clone());

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
                        .ok_or(ErrorResponse::new_status(400, "content is null"))?,
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
    let mut follow_service = new_follow_service(pool.clone(), data.config().clone());

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
    let mut follow_service = new_follow_service(pool.clone(), data.config().clone());

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
        "services": {"inbound": [], "outbound": []},
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

#[post("/user/{user_spec}/inbox")]
async fn user_inbox(
    params: web::Path<UserChooseParams>,
    app: web::Data<AppState>,
    auth: AuthUser,
    body: web::Json<serde_json::Value>,
) -> HandlerResponse<impl Responder> {
    debug!("user_inbox: {:?}", params);
    debug!("{:?}", body);

    let mut authed_user = auth.must_auth()?;

    // deserialize into ActivityPub activity
    let activity = models::apub::Activity::deserialize(&body.0).map_err(|e| {
        warn!("Failed to deserialize activity: {:?}", e);
        ErrorResponse::new_status(400, "invalid activity")
    })?;

    debug!("parsed activity {:#?}", activity);
    use models::apub::Activity::*;
    match activity {
        Accept(a) => {
            let actor_id = a.actor;
            let req_spec = match a.object {
                models::apub::IdOrObject::Id(id) => todo!("fetch object by id: {}", id),
                models::apub::IdOrObject::Object(obj) => {
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
            };

            let mut follow_service = new_follow_service(app.pool().clone(), app.config().clone());
            info!("accepting follow request of {:#?}", req_spec);
            let result = follow_service.follow_request_accepted(&req_spec).await?;
            info!(
                "follow request accepted: FollowRequestID: {} -> {}",
                result.follower_id.simple(),
                result.followee_id.simple()
            );
        }
        Follow(follow) => {
            let follow_id = follow.id;
            let actor_id = follow.actor;
            let object_id = follow.object.get_id().to_string();

            debug!("accepting follow request of {} -> {}", actor_id, object_id);
            let mut follow_service = new_follow_service(app.pool().clone(), app.config().clone());
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
            // get object
            let object = {
                match create.object {
                    IdOrObject::Id(id) => {
                        // request object using id
                        let mut reqester_service = new_apub_reqwester_service();
                        let CreatableObject::Note(object_note) =
                            reqester_service.fetch_post(&id).await?;
                        Some(object_note)
                    }
                    IdOrObject::Object(CreatableObject::Note(note)) => Some(note),
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

            let mut post_service =
                new_post_create_service(app.pool().clone(), app.config().clone());
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

            let mut post_service =
                new_post_create_service(app.pool().clone(), app.config().clone());
            let result = post_service.create_post(&repost).await?;
            info!("repost created: {}", result);
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

#[actix_web::main]
async fn main() -> std::io::Result<()> {
    tracing_subscriber::fmt()
        .with_max_level(tracing::Level::DEBUG)
        .with_test_writer()
        .init();

    let mut file = std::fs::File::open("lightpub.yml.sample").unwrap();
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
        .max_connections(5)
        .connect(&conn_str)
        .await
        .expect("connect to database");
    tracing::info!("Connected to database");

    let app_state = state::AppState::new(pool, config.clone());

    #[derive(OpenApi)]
    #[openapi(
        paths(login, register, post_post),
        components(schemas(
            LoginResponse,
            LoginBody,
            RegisterBody,
            RegisterResponse,
            PostRequest,
            PostCreateResponse
        ))
    )]
    struct ApiDoc1;

    HttpServer::new(move || {
        App::new()
            .app_data(web::Data::new(app_state.clone()))
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
            .service(SwaggerUi::new("/swagger-ui/{_:.*}").urls(vec![(
                utoipa_swagger_ui::Url::new("api1", "/api-docs/openapi1.json"),
                ApiDoc1::openapi(),
            )]))
    })
    .bind(("127.0.0.1", 8000))?
    .run()
    .await
}
