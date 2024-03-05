mod config;
pub mod models;
pub mod services;
pub mod state;
pub mod utils;

use crate::services::{
    PostCreateRequest, PostCreateRequestBuilder, UserAuthService, UserCreateService,
};
use actix_web::{
    dev::ServiceRequest, get, post, web, App, FromRequest, HttpResponse, HttpServer, Responder,
};
use actix_web_httpauth::{extractors::bearer::BearerAuth, middleware::HttpAuthentication};
use config::Config;
use models::{PostPrivacy, User};
use serde::{Deserialize, Serialize};
use services::{
    db::{new_auth_service, new_local_user_finder_service, post::new_post_create_service},
    AuthError, PostCreateService, ServiceError, UserCreateRequest, UserCreateRequestBuilder,
    UserLoginError, UserLoginRequest, UserLoginRequestBuilder,
};
use sqlx::{mysql::MySqlPoolOptions, MySqlPool};
use state::AppState;
use std::{
    fmt::{Debug, Display, Formatter},
    future::Future,
    io::Read,
    pin::Pin,
};
use tracing;
use utils::user::UserSpecifier;
use uuid::{fmt::Simple, Uuid};

use crate::services::db::new_user_service;
use utoipa::OpenApi;
use utoipa::ToSchema;
use utoipa_swagger_ui::SwaggerUi;

async fn validator(
    req: ServiceRequest,
    _credentials: BearerAuth,
) -> Result<ServiceRequest, (actix_web::Error, ServiceRequest)> {
    Ok(req)
}

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

impl FromRequest for AuthUser {
    type Error = ErrorResponse;
    type Future = Pin<Box<dyn Future<Output = Result<AuthUser, Self::Error>>>>;

    fn from_request(
        req: &actix_web::HttpRequest,
        payload: &mut actix_web::dev::Payload,
    ) -> Self::Future {
        let req = req.clone();
        Box::pin(async move {
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
                tracing::error!("Specific error not handled: {:?}", &e);
                ErrorResponse::new_status(500, "internal server error")
            }
            ServiceError::MiscError(e) => {
                tracing::error!("Misc error: {:?}", &e);
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
    tracing::error!("Internal server error: {:?}", &error);
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
    pub content: String,
    pub privacy: PostPrivacy,
    pub reply_to_id: Option<Uuid>,
    pub repost_of_id: Option<Uuid>,
}

#[derive(Debug, Serialize, ToSchema)]
pub struct PostCreateResponse {}

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
    let mut post_service =
        new_post_create_service(pool.clone(), new_local_user_finder_service(pool));

    let req = post_service
        .create_post(
            &PostCreateRequestBuilder::default()
                .poster(UserSpecifier::from_id(user.id))
                .content(body.content.clone())
                .privacy(body.privacy.clone())
                .build()
                .unwrap(),
        )
        .await?;

    Ok(HttpResponse::Ok().json(PostCreateResponse {}))
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
    let conn_str = (format!(
        "mysql://{}:{}@{}:{}/{}",
        config.database.user,
        config.database.password,
        config.database.host,
        config.database.port,
        config.database.name
    ));
    let pool = MySqlPoolOptions::new()
        .max_connections(5)
        .connect(&conn_str)
        .await
        .expect("connect to database");
    tracing::info!("Connected to database");

    let app_state = state::AppState::new(pool);

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
            .service(register)
            .service(login)
            .service(post_post)
            .service(SwaggerUi::new("/swagger-ui/{_:.*}").urls(vec![(
                utoipa_swagger_ui::Url::new("api1", "/api-docs/openapi1.json"),
                ApiDoc1::openapi(),
            )]))
    })
    .bind(("127.0.0.1", 8000))?
    .run()
    .await
}
