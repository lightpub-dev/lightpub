mod config;
pub mod models;
pub mod services;
pub mod state;
pub mod utils;

use actix_web::{get, post, web, App, HttpResponse, HttpServer, Responder};
use config::Config;
use serde::{Deserialize, Serialize};
use sqlx::mysql::MySqlPoolOptions;
use state::AppState;
use std::{
    fmt::{Display, Formatter},
    io::Read,
};
use tracing;
use uuid::Uuid;

#[get("/api/")]
async fn hello() -> impl Responder {
    HttpResponse::Ok().body("Hello world!")
}

#[post("/echo")]
async fn echo(req_body: String) -> impl Responder {
    HttpResponse::Ok().body(req_body)
}

#[derive(Debug, Serialize)]
struct ErrorResponse {
    message: String,
}

impl Display for ErrorResponse {
    fn fmt(&self, f: &mut Formatter<'_>) -> std::fmt::Result {
        write!(f, "{}", self.message)
    }
}

impl actix_web::ResponseError for ErrorResponse {
    fn status_code(&self) -> actix_web::http::StatusCode {
        actix_web::http::StatusCode::BAD_REQUEST
    }
}

#[derive(Debug, Deserialize)]
struct RegisterBody {
    pub username: String,
    pub nickname: String,
    pub password: String,
}

impl Into<services::user::UserCreateRequest> for RegisterBody {
    fn into(self) -> services::user::UserCreateRequest {
        services::user::UserCreateRequest::new(self.username, self.nickname, self.password)
    }
}

#[derive(Debug, Serialize)]
struct RegisterResponse {
    user_id: Uuid,
}

#[post("/register")]
async fn register(
    body: web::Json<RegisterBody>,
    data: web::Data<AppState>,
) -> Result<impl Responder, ErrorResponse> {
    let mut us = services::new_user_service(data.pool().clone());
    let req = us.register_local_user(body.0.into()).await.unwrap();
    Ok(HttpResponse::Ok().json(RegisterResponse { user_id: req }))
}

#[post("/login")]
async fn login() -> impl Responder {}

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

    HttpServer::new(move || {
        App::new()
            .app_data(web::Data::new(app_state.clone()))
            .service(hello)
            .service(echo)
            .service(register)
            .route("/hey", web::get().to(manual_hello))
    })
    .bind(("127.0.0.1", 8000))?
    .run()
    .await
}
