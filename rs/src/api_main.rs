use lightpub::api::model::{
    LoginRequest, PostCreateRequest, PostCreateResponse, RegisterRequest, RegisterResponse,
    UserSpecifier, WebfingerQuery,
};
use lightpub::api::{AppConfig, AppState, ErrorResponse};
use lightpub::application::service::post::{
    NormalPostCreateCommand, QuoteCreateCommand, ReplyPostCreateCommand, RepostCreateCommand,
};
use lightpub::application::service::user::GetUserOptionsBuilder;
use lightpub::domain::model::post::PostPrivacy;
use lightpub::domain::model::user::User;
use lightpub::domain::service::user::UserService;

use actix_cors::Cors;
use actix_multipart::form::MultipartForm;
use actix_web::http::header;
use actix_web::{
    delete, get, middleware::Logger, post, put, web, App, FromRequest, HttpResponse, HttpServer,
    Responder,
};
use clap::Parser;
use serde::{Deserialize, Serialize};
use serde_json::json;
use sqlx::sqlite::SqlitePoolOptions;
use std::borrow::BorrowMut;
use std::path::PathBuf;
use std::{
    fmt::{Debug, Display, Formatter},
    future::Future,
    io::Read,
    pin::Pin,
};
use tracing::{debug, error, info, warn};
use uuid::{fmt::Simple, Uuid};

#[allow(unreachable_code)]
fn api_todo() -> HandlerResponse<impl Responder> {
    todo!() as HandlerResponse<String>
}

#[derive(Debug)]
struct AuthUser {
    authed_user: Option<String>, // user_id
}

impl AuthUser {
    pub fn from_user(user: impl Into<String>) -> Self {
        Self {
            authed_user: Some(user.into()),
        }
    }

    pub fn unauthed() -> Self {
        Self { authed_user: None }
    }

    pub fn must_auth(&self) -> Result<&str, ErrorResponse> {
        match &self.authed_user {
            Some(u) => Ok(&u),
            None => Err(ErrorResponse::new(401, "unauthorized")),
        }
    }

    pub fn may_auth(&self) -> Result<Option<&str>, ErrorResponse> {
        Ok(self.authed_user.as_ref().map(|s| s.as_str()))
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

            let mut user_security_service = data.user_security_service();

            let authed_user = user_security_service.validate_token(bearer).await?;

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

#[post("/register")]
async fn register(
    body: web::Json<RegisterRequest>,
    data: web::Data<AppState>,
) -> Result<impl Responder, ErrorResponse> {
    // TODO: open registration check
    // if !data.config().instance.open_registration {
    //     return Err(ErrorResponse::new_status(403, "registration is closed"));
    // }

    let mut user_service = data.user_service();

    let user = user_service
        .create_user(&body.username, &body.nickname, &body.password)
        .await;

    match user {
        Ok(user_id) => Ok(HttpResponse::Ok().json(RegisterResponse {
            user_id: user_id.user_id().to_string(),
        })),
        Err(e) => {
            panic!("Failed to create user: {:?}", e);
        }
    }
}

#[derive(Debug, Serialize)]
struct LoginResponse {
    token: String,
}

#[post("/login")]
async fn login(
    body: web::Json<LoginRequest>,
    data: web::Data<AppState>,
) -> Result<impl Responder, ErrorResponse> {
    let mut user_security_service = data.user_security_service();

    let result = user_security_service
        .login(&body.username, &body.password)
        .await;

    match result {
        Ok(token) => Ok(HttpResponse::Ok().json(LoginResponse {
            token: token.token().to_string(),
        })),
        Err(e) => {
            panic!("Failed to login: {:?}", e);
        }
    }
}

fn post_privacy_from_str(s: &str) -> Result<PostPrivacy, ErrorResponse> {
    match s {
        "public" => Ok(PostPrivacy::Public),
        "unlisted" => Ok(PostPrivacy::Unlisted),
        "followers" => Ok(PostPrivacy::Followers),
        "private" => Ok(PostPrivacy::Private),
        _ => Err(ErrorResponse::new(400, "invalid privacy")),
    }
}

#[post("/post")]
async fn post_post(
    body: web::Json<PostCreateRequest>,
    data: web::Data<AppState>,
    auth: AuthUser,
) -> Result<impl Responder, ErrorResponse> {
    let user = auth.must_auth()?;

    let mut post_create_service = data.post_create_service();

    let author_id = user;
    let post = match (&body.repost_of_id, &body.reply_to_id) {
        (None, None) => post_create_service
            .create_post(&NormalPostCreateCommand {
                author_id: &author_id,
                content: body.content.as_ref().unwrap(),
                privacy: post_privacy_from_str(&body.privacy)?,
                created_at: chrono::Utc::now(),
            })
            .await
            .unwrap(),
        (Some(repost_of_id), None) => {
            if let Some(content) = body.content.clone() {
                post_create_service
                    .create_quote(&QuoteCreateCommand {
                        author_id: &author_id,
                        content: &content,
                        privacy: post_privacy_from_str(&body.privacy)?,
                        quote_of: &repost_of_id,
                        created_at: chrono::Utc::now(),
                    })
                    .await
                    .unwrap()
            } else {
                post_create_service
                    .create_repost(&RepostCreateCommand {
                        author_id: &author_id,
                        privacy: post_privacy_from_str(&body.privacy)?,
                        repost_of: &repost_of_id,
                        created_at: chrono::Utc::now(),
                    })
                    .await
                    .unwrap()
            }
        }
        (None, Some(reply_to_id)) => post_create_service
            .create_reply(&ReplyPostCreateCommand {
                author_id: &author_id,
                content: body.content.as_ref().unwrap(),
                privacy: post_privacy_from_str(&body.privacy)?,
                reply_to: &reply_to_id,
                created_at: chrono::Utc::now(),
            })
            .await
            .unwrap(),
        _ => {
            return Err(ErrorResponse::new(
                400,
                "repost_of_id and reply_to_id cannot be set at the same time",
            ))
        }
    };

    Ok(HttpResponse::Ok().json(PostCreateResponse {
        post_id: post.id().to_string(),
    }))
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

    let mut user_service = data.user_service();
    let mut follow_service = data.follow_service();

    let followee_id = match &path.user_spec {
        UserSpecifier::ID(id) => {
            if !user_service.id_exists(&id).await? {
                return Err(ErrorResponse::new(404, "user not found"));
            }
            id.to_string()
        }
        UserSpecifier::Username(username, host) => {
            match user_service
                .get_user_id_by_username_and_host(&username, host.as_ref().map(|s| s.as_str()))
                .await?
            {
                None => return Err(ErrorResponse::new(404, "user not found")),
                Some(id) => id.user_id().to_string(),
            }
        }
    };

    follow_service.follow(user, &followee_id).await?;

    Ok(HttpResponse::Ok().finish())
}

#[delete("/user/{user_spec}/follow")]
async fn user_delete_follow(
    path: web::Path<UserChooseParams>,
    data: web::Data<AppState>,
    auth: AuthUser,
) -> Result<impl Responder, ErrorResponse> {
    let user = auth.must_auth()?;

    let mut user_service = data.user_service();
    let mut follow_service = data.follow_service();

    let followee_id = match &path.user_spec {
        UserSpecifier::ID(id) => {
            if !user_service.id_exists(&id).await? {
                return Err(ErrorResponse::new(404, "user not found"));
            }
            id.to_string()
        }
        UserSpecifier::Username(username, host) => {
            match user_service
                .get_user_id_by_username_and_host(&username, host.as_ref().map(|s| s.as_str()))
                .await?
            {
                None => return Err(ErrorResponse::new(404, "user not found")),
                Some(id) => id.user_id().to_string(),
            }
        }
    };

    follow_service.unfollow(user, &followee_id).await?;

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

fn gen_node_info(node_info_version: &str, config: &AppConfig) -> serde_json::Value {
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
        // "metadata": {
        //     "nodeName": config.instance.name,
        //     "nodeDescription": config.instance.description,
        // },
    })
}

#[get("/.well-known/nodeinfo")]
async fn well_known_node_info(app: web::Data<AppState>) -> impl Responder {
    let link_2_0 = format!("{}/nodeinfo/2.0", app.config().base_url());
    let link_2_1 = format!("{}/nodeinfo/2.1", app.config().base_url());
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

    let mut user_service = app.user_service();

    let acct_id = parts[1];
    let user_id = if !acct_id.contains("@") {
        // contains username only
        user_service
            .get_user_id_by_username_and_host(acct_id, None)
            .await?
    } else {
        let parts: Vec<&str> = acct_id.split("@").collect();
        if parts.len() != 2 {
            return Ok(HttpResponse::BadRequest().body("Invalid resource"));
        }
        if parts[1] != app.config().hostname() {
            return Ok(HttpResponse::NotFound().body("user not found"));
        }
        user_service
            .get_user_id_by_username_and_host(parts[0], None)
            .await?
    };

    let user = match user_id {
        Some(id) => user_service
            .get_user_by_id(
                &id.user_id().to_string(),
                &GetUserOptionsBuilder::default()
                    .fill_uris(true) // set uri field event if the user is local
                    .build()
                    .unwrap(),
            )
            .await?
            .unwrap(), // user should exist
        None => return Ok(HttpResponse::NotFound().body("user not found")),
    };

    Ok(HttpResponse::Ok().content_type("application/json").body(
        json!({
            "subject": acct_id,
            "links": [
                {
                    "rel": "self",
                    "type": "application/activity+json",
                    "href": format!("{}/user/{}", app.config().base_url(), user.uri().unwrap())
                }
            ]
        })
        .to_string(),
    ))
}

#[get("/.well-known/host-meta")]
async fn host_meta(app: web::Data<AppState>) -> HandlerResponse<impl Responder> {
    let base_url = app.config().base_url();
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
    api_todo()
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
    api_todo()
}

#[post("/user/{user_spec}/outbox")]
async fn post_user_outbox(app: web::Data<AppState>) -> HandlerResponse<impl Responder> {
    api_todo()
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
    api_todo()
}

#[post("/debug/truncate")]
async fn truncate_database(app: web::Data<AppState>) -> impl Responder {
    let table_names = [
        "users",
        "posts",
        "uploaded_files",
        "user_follows",
        "user_labels",
        "user_tokens",
        "post_attachments",
        "post_favorites",
        "post_hashtags",
        "post_mentions",
        "reactions",
        "post_reactions",
        "user_follow_requests",
        "remote_user_details",
        "user_keys",
        "remote_users",
    ];
    for table_name in table_names {
        match sqlx::query("DELETE FROM users").execute(app.pool()).await {
            Ok(_) => {
                info!("Truncated table: {}", table_name);
            }
            Err(e) => {
                error!("Failed to truncate table {}: {:?}", table_name, e);
                return HttpResponse::InternalServerError().finish();
            }
        }
    }

    HttpResponse::Ok().finish()
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
    let conn_str = format!("sqlite:{}", config.database.path);
    let pool = SqlitePoolOptions::new()
        .connect(&conn_str)
        .await
        .expect("connect to database");
    tracing::info!("Connected to database");

    tracing::info!("Running migrations");
    sqlx::migrate!("./migrations")
        .run(&pool)
        .await
        .expect("run migrations");
    tracing::info!("Migrations complete");

    // create upload_dir
    let upload_dir = config.upload_dir.clone();
    web::block(move || {
        std::fs::create_dir_all(upload_dir).expect("failed to create upload_dir");
    })
    .await
    .unwrap();

    let app_state = AppState::new();

    HttpServer::new(move || {
        let cors = Cors::default()
            .allowed_origin("http://localhost:5173")
            .allowed_methods(vec!["GET", "POST", "PUT", "DELETE", "PATCH"])
            .allowed_headers(vec![
                header::AUTHORIZATION,
                header::CONTENT_TYPE,
                header::ACCEPT,
            ]);

        let mut app = App::new()
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
            .service(get_user_outbox)
            .service(timeline);

        if config.dev.debug {
            app = app.service(truncate_database);
        }

        app
    })
    .bind(("0.0.0.0", 8000))?
    .run()
    .await
}
