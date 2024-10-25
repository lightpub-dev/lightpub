use crate::api_root::{
    add_post_bookmark, add_post_favorite, delete_post_bookmark, delete_post_favorite,
    delete_single_post, file_upload, get_single_post, get_user_followers, get_user_following,
    get_user_outbox, get_user_posts, host_meta, login, modify_post_reaction, node_info_2_0,
    node_info_2_1, post_post, register, timeline, truncate_database, update_my_profile,
    user_create_follow, user_delete_follow, user_get, user_inbox, webfinger, well_known_node_info,
};
use std::io::Read;

use actix_http::Request;
use actix_web::body;
use actix_web::error::Error;
use actix_web::{
    body::MessageBody,
    dev::{Service, ServiceFactory, ServiceRequest, ServiceResponse},
    test::{call_service, init_service, TestRequest},
    web, App,
};
use reqwest::Method;
use serde::Deserialize;
use serde_json::json;
use sqlx::sqlite::SqlitePoolOptions;
use std::fmt::Debug;

use crate::{api::state::AppState, config::Config};

async fn setup_server() -> App<
    impl ServiceFactory<
        ServiceRequest,
        Config = (),
        Response = ServiceResponse<impl MessageBody + std::fmt::Debug>,
        Error = actix_web::Error,
        InitError = (),
    >,
> {
    let _ = tracing_subscriber::fmt()
        .with_max_level(tracing::Level::DEBUG)
        .with_test_writer()
        .try_init();

    let mut file = std::fs::File::open("lightpub.test.yml").unwrap();
    let mut contents = String::new();
    file.read_to_string(&mut contents)
        .expect("Unable to read file");
    let mut config: Config = serde_yaml::from_str(&contents).expect("Unable to deserialize YAML");

    // create temporary db
    // let temp_di = tempdir().expect("create temp dir");
    // let temp_db = temp_dir.path().join("temp_db.sqlite3");
    // let temp_db_file = File::create(&temp_db).await.expect("create temp db");
    // drop(temp_db_file);
    config.database.path = ":memory:".to_string();

    // connect to db
    let conn_str = format!("sqlite:{}", config.database.path);
    let pool = SqlitePoolOptions::new()
        .idle_timeout(std::time::Duration::from_secs(60 * 60))
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
    // let upload_dir = config.upload_dir.clone();
    // web::block(move || {
    //     std::fs::create_dir_all(upload_dir).expect("failed to create upload_dir");
    // })
    // .await
    // .unwrap();

    let app_state = AppState::new(pool, config.clone());

    let app = App::new()
        .app_data(web::Data::new(app_state.clone()))
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
        .service(truncate_database);

    app
}

async fn init_app(
) -> impl Service<Request, Response = ServiceResponse<impl MessageBody + Debug>, Error = Error> {
    init_service(setup_server().await).await
}

#[actix_web::test]
async fn auth_register() {
    let srv = setup_server().await;
    let app = init_service(srv).await;
    let req = TestRequest::default()
        .uri("/register")
        .method(Method::POST)
        .set_json(json!({
            "username": "admin",
            "nickname": "admin dayo",
            "password": "1234Abcd!"
        }))
        .to_request();
    let resp = call_service(&app, req).await;
    assert_eq!(resp.status(), 200);
}

async fn register_user(
    app: impl Service<Request, Response = ServiceResponse<impl MessageBody + Debug>, Error = Error>,
    username: &str,
    nickname: &str,
    password: &str,
) {
    let req = TestRequest::default()
        .uri("/register")
        .method(Method::POST)
        .set_json(json!({
            "username": username,
            "nickname": nickname,
            "password": password
        }))
        .to_request();
    let resp = call_service(&app, req).await;
    assert_eq!(resp.status(), 200);
}

async fn register_admin(
    app: impl Service<Request, Response = ServiceResponse<impl MessageBody + Debug>, Error = Error>,
) {
    register_user(app, "admin", "admin dayo", "1234Abcd!").await
}

#[actix_web::test]
async fn auth_login_success() {
    let app = init_app().await;
    register_admin(&app).await;

    let req = TestRequest::default()
        .uri("/login")
        .method(Method::POST)
        .set_json(json!({
            "username": "admin",
            "password": "1234Abcd!"
        }))
        .to_request();
    let resp = call_service(&app, req).await;
    assert_eq!(resp.status(), 200);
}

#[actix_web::test]
async fn auth_login_fail() {
    let app = init_app().await;
    register_admin(&app).await;

    let req = TestRequest::default()
        .uri("/login")
        .method(Method::POST)
        .set_json(json!({
            "username": "admin",
            "password": "1234Abcd"
        }))
        .to_request();
    let resp = call_service(&app, req).await;
    assert_eq!(resp.status(), 401);
}

#[actix_web::test]
async fn auth_register_duplicated() {
    let app = init_app().await;

    register_user(&app, "admin", "admin dayo", "1234Abcd!").await;

    let req = TestRequest::default()
        .uri("/register")
        .method(Method::POST)
        .set_json(json!({
            "username": "admin",
            "nickname": "admin dayo2",
            "password": "1234Abcd?"
        }))
        .to_request();
    let resp = call_service(&app, req).await;
    assert_eq!(resp.status(), 400);
}

async fn auth_bad_username_check(
    app: impl Service<Request, Response = ServiceResponse<impl MessageBody + Debug>, Error = Error>,
    username: &str,
) {
    let body = json!({
        "username": username,
        "nickname": username,
        "password": "1234Abcd!"
    });
    let req = TestRequest::default()
        .uri("/register")
        .method(Method::POST)
        .set_json(body)
        .to_request();
    let resp = call_service(&app, req).await;
    assert_eq!(resp.status(), 400);
}

#[actix_web::test]
async fn auth_bad_username_kanji() {
    let app = init_app().await;
    auth_bad_username_check(app, "kanji感じ").await;
}

#[actix_web::test]
async fn auth_bad_username_too_short() {
    let app = init_app().await;
    auth_bad_username_check(app, "ab").await;
}

#[actix_web::test]
async fn auth_bad_username_too_long() {
    let app = init_app().await;
    auth_bad_username_check(app, "123456789abcdefgh").await;
}

#[actix_web::test]
async fn auth_bad_username_special_chars() {
    let app = init_app().await;
    auth_bad_username_check(app, "special!char@foobar").await;
}

#[actix_web::test]
async fn auth_login_non_existent_user() {
    let app = init_app().await;

    let req = TestRequest::default()
        .uri("/login")
        .method(Method::POST)
        .set_json(json!({
            "username": "foo",
            "password": "1234Abcd!"
        }))
        .to_request();
    let resp = call_service(&app, req).await;
    assert_eq!(resp.status(), 401);
}

const GOOD_PASSWORD: &str = "1234Abcd!";

async fn parse_body<T: serde::de::DeserializeOwned, B: MessageBody>(
    body: B,
) -> Result<T, anyhow::Error> {
    let bytes = body::to_bytes(body).await.ok().expect("to_bytes");
    let body = bytes.as_ref();
    let body = std::str::from_utf8(body).unwrap();
    serde_json::from_str(body).map_err(|e| e.into())
}

#[derive(Deserialize, Debug)]
struct LoginResponse {
    token: String,
}

async fn login_user(
    app: impl Service<Request, Response = ServiceResponse<impl MessageBody + Debug>, Error = Error>,
    username: &str,
    password: &str,
) -> Result<LoginResponse, anyhow::Error> {
    let req = TestRequest::default()
        .uri("/login")
        .method(Method::POST)
        .set_json(json!({
            "username": username,
            "password": password
        }))
        .to_request();
    let resp = call_service(&app, req).await;
    assert_eq!(resp.status(), 200);

    let resp: LoginResponse = parse_body(resp.into_body()).await.unwrap();
    assert_ne!(resp.token, "");

    Ok(resp)
}

fn make_auth_header(token: &str) -> String {
    format!("Bearer {}", token)
}

trait AuthAttachExt {
    type R;
    fn attach_token(self, token: &str) -> Self::R;
}

impl AuthAttachExt for TestRequest {
    type R = TestRequest;
    fn attach_token(self, token: &str) -> Self::R {
        self.append_header(("Authorization", make_auth_header(token.as_ref())))
    }
}

#[actix_web::test]
async fn post_public() {
    let app = init_app().await;
    register_user(&app, "testuser", "testuser", GOOD_PASSWORD).await;

    let token = login_user(&app, "testuser", GOOD_PASSWORD).await.unwrap();

    let req = TestRequest::default()
        .uri("/post")
        .method(Method::POST)
        .attach_token(&token.token)
        .set_json(json!({
            "content": "public content",
            "privacy": "public"
        }))
        .to_request();

    let resp = call_service(&app, req).await;
    assert_eq!(resp.status(), 200);
}

#[actix_web::test]
async fn post_unlisted() {
    let app = init_app().await;
    register_user(&app, "testuser", "testuser", GOOD_PASSWORD).await;

    let token = login_user(&app, "testuser", GOOD_PASSWORD).await.unwrap();

    let req = TestRequest::default()
        .uri("/post")
        .method(Method::POST)
        .attach_token(&token.token)
        .set_json(json!({
            "content": "unlisted content",
            "privacy": "unlisted"
        }))
        .to_request();

    let resp = call_service(&app, req).await;
    assert_eq!(resp.status(), 200);
}

#[actix_web::test]
async fn post_follower() {
    let app = init_app().await;
    register_user(&app, "testuser", "testuser", GOOD_PASSWORD).await;

    let token = login_user(&app, "testuser", GOOD_PASSWORD).await.unwrap();

    let req = TestRequest::default()
        .uri("/post")
        .method(Method::POST)
        .attach_token(&token.token)
        .set_json(json!({
            "content": "follower-only content",
            "privacy": "follower"
        }))
        .to_request();

    let resp = call_service(&app, req).await;
    assert_eq!(resp.status(), 200);
}

#[actix_web::test]
async fn post_private() {
    let app = init_app().await;
    register_user(&app, "testuser", "testuser", GOOD_PASSWORD).await;

    let token = login_user(&app, "testuser", GOOD_PASSWORD).await.unwrap();

    let req = TestRequest::default()
        .uri("/post")
        .method(Method::POST)
        .attach_token(&token.token)
        .set_json(json!({
            "content": "private content",
            "privacy": "private"
        }))
        .to_request();

    let resp = call_service(&app, req).await;
    assert_eq!(resp.status(), 200);
}

#[derive(Debug, Deserialize)]
struct PostCreateResponse {
    post_id: String,
}

#[actix_web::test]
async fn post_delete_my_post() {
    let app = init_app().await;
    register_user(&app, "testuser", "testuser", GOOD_PASSWORD).await;

    let token = login_user(&app, "testuser", GOOD_PASSWORD).await.unwrap();

    let req = TestRequest::default()
        .uri("/post")
        .method(Method::POST)
        .attach_token(&token.token)
        .set_json(json!({
            "content": "public content",
            "privacy": "public"
        }))
        .to_request();

    let resp = call_service(&app, req).await;
    assert_eq!(resp.status(), 200);
    let resp: PostCreateResponse = parse_body(resp.into_body()).await.unwrap();

    let req = TestRequest::default()
        .uri(&format!("/post/{}", resp.post_id))
        .method(Method::DELETE)
        .attach_token(&token.token)
        .to_request();

    let resp = call_service(&app, req).await;
    assert_eq!(resp.status(), 200);
}

#[actix_web::test]
async fn post_delete_non_existent_post() {
    let app = init_app().await;
    register_user(&app, "testuser", "testuser", GOOD_PASSWORD).await;

    let token = login_user(&app, "testuser", GOOD_PASSWORD).await.unwrap();

    let req = TestRequest::default()
        .uri("/post/1234")
        .method(Method::DELETE)
        .attach_token(&token.token)
        .to_request();

    let resp = call_service(&app, req).await;
    assert_eq!(resp.status(), 404);
}
