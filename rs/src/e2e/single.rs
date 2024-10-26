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
    let random_str = uuid::Uuid::new_v4().to_string();
    config.database.path = format!("file:{}?mode=memory&cache=shared", random_str);

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

#[derive(Debug)]
struct PostReplySetup {
    token: String,
    public_post_id: String,
    follower_post_id: String,
    private_post_id: String,
}

async fn post_reply_setup(
    app: impl Service<Request, Response = ServiceResponse<impl MessageBody + Debug>, Error = Error>,
) -> Result<PostReplySetup, anyhow::Error> {
    register_user(&app, "testuser", "testuser", GOOD_PASSWORD).await;
    register_user(&app, "testuser2", "testuser2", GOOD_PASSWORD).await;
    let login_resp = login_user(&app, "testuser", GOOD_PASSWORD).await.unwrap();
    let login_resp2 = login_user(&app, "testuser2", GOOD_PASSWORD).await.unwrap();

    let get_post_id = {
        let login_resp_token = &login_resp2.token;
        let app_ref = &app;
        move |content: &str, privacy: &str| {
            let req_body = json!({
                "content": content,
                "privacy": privacy
            });
            async move {
                let req = TestRequest::default()
                    .uri("/post")
                    .method(Method::POST)
                    .attach_token(login_resp_token)
                    .set_json(req_body)
                    .to_request();
                let resp = call_service(app_ref, req).await;
                assert_eq!(resp.status(), 200);
                let body: PostCreateResponse = parse_body(resp.into_body()).await.unwrap();
                body.post_id.clone()
            }
        }
    };

    let public_post_id = get_post_id("public parent post", "public").await;
    let follower_post_id = get_post_id("follower parent post", "follower").await;
    let private_post_id = get_post_id("private parent post", "private").await;

    Ok(PostReplySetup {
        token: login_resp.token,
        public_post_id,
        follower_post_id,
        private_post_id,
    })
}

#[actix_web::test]
async fn post_reply_to_public() {
    let app = init_app().await;
    let setup = post_reply_setup(&app).await.unwrap();

    let req = TestRequest::default()
        .uri("/post")
        .method(Method::POST)
        .set_json(json!({
            "content": "reply to public",
            "privacy": "public",
            "reply_to_id": &setup.public_post_id
        }))
        .attach_token(&setup.token)
        .to_request();
    let resp = call_service(&app, req).await;
    assert_eq!(resp.status(), 200)
}

#[actix_web::test]
async fn post_reply_to_follower_public_by_non_follower() {
    let app = init_app().await;
    let setup = post_reply_setup(&app).await.unwrap();

    let req = TestRequest::default()
        .uri("/post")
        .method(Method::POST)
        .set_json(json!({
            "content": "reply to follower-only",
            "privacy": "public",
            "reply_to_id": &setup.follower_post_id
        }))
        .attach_token(&setup.token)
        .to_request();
    let resp = call_service(&app, req).await;
    assert_eq!(resp.status(), 404)
}

#[actix_web::test]
async fn post_reply_to_private_public() {
    let app = init_app().await;
    let setup = post_reply_setup(&app).await.unwrap();

    let req = TestRequest::default()
        .uri("/post")
        .method(Method::POST)
        .set_json(json!({
            "content": "reply to private",
            "privacy": "public",
            "reply_to_id": &setup.private_post_id
        }))
        .attach_token(&setup.token)
        .to_request();
    let resp = call_service(&app, req).await;
    assert_eq!(resp.status(), 404)
}

#[actix_web::test]
async fn post_reply_to_public_private() {
    let app = init_app().await;
    let setup = post_reply_setup(&app).await.unwrap();

    let req = TestRequest::default()
        .uri("/post")
        .method(Method::POST)
        .set_json(json!({
            "content": "reply to private",
            "privacy": "private",
            "reply_to_id": &setup.public_post_id
        }))
        .attach_token(&setup.token)
        .to_request();
    let resp = call_service(&app, req).await;
    assert_eq!(resp.status(), 200)
}

#[derive(Debug, Clone)]
struct PostRepostSetup {
    token: String,
    other_public_post_id: String,
    other_follower_post_id: String,
    other_private_post_id: String,
    my_public_post_id: String,
    my_follower_post_id: String,
    my_private_post_id: String,
}

async fn post_repost_setup(
    app: impl Service<Request, Response = ServiceResponse<impl MessageBody + Debug>, Error = Error>,
) -> Result<PostRepostSetup, anyhow::Error> {
    register_user(&app, "testuser", "testuser", GOOD_PASSWORD).await;
    register_user(&app, "testuser2", "testuser2", GOOD_PASSWORD).await;
    let login_resp = login_user(&app, "testuser", GOOD_PASSWORD).await.unwrap();
    let login_resp2 = login_user(&app, "testuser2", GOOD_PASSWORD).await.unwrap();

    let get_post_id = {
        let app_ref = &app;
        move |token: &str, content: &str, privacy: &str| {
            let req_body = json!({
                "content": content,
                "privacy": privacy
            });
            let token = token.to_string();
            async move {
                let req = TestRequest::default()
                    .uri("/post")
                    .method(Method::POST)
                    .attach_token(&token)
                    .set_json(req_body)
                    .to_request();
                let resp = call_service(app_ref, req).await;
                assert_eq!(resp.status(), 200);
                let body: PostCreateResponse = parse_body(resp.into_body()).await.unwrap();
                body.post_id.clone()
            }
        }
    };

    let my_public_post_id = get_post_id(&login_resp.token, "public parent post", "public").await;
    let my_follower_post_id =
        get_post_id(&login_resp.token, "follower parent post", "follower").await;
    let my_private_post_id = get_post_id(&login_resp.token, "private parent post", "private").await;

    let other_public_post_id =
        get_post_id(&login_resp2.token, "public parent post", "public").await;
    let other_follower_post_id =
        get_post_id(&login_resp2.token, "follower parent post", "follower").await;
    let other_private_post_id =
        get_post_id(&login_resp2.token, "private parent post", "private").await;

    Ok(PostRepostSetup {
        token: login_resp.token,
        my_public_post_id,
        my_follower_post_id,
        my_private_post_id,
        other_public_post_id,
        other_follower_post_id,
        other_private_post_id,
    })
}

#[actix_web::test]
async fn post_repost_of_public_public() {
    let app = init_app().await;
    let setup = post_repost_setup(&app).await.unwrap();

    let req = TestRequest::default()
        .uri("/post")
        .method(Method::POST)
        .set_json(json!({
            "privacy": "public",
            "repost_of_id": &setup.other_public_post_id
        }))
        .attach_token(&setup.token)
        .to_request();

    let resp = call_service(&app, req).await;
    assert_eq!(resp.status(), 200);
}

#[actix_web::test]
async fn post_repost_of_follower_public() {
    let app = init_app().await;
    let setup = post_repost_setup(&app).await.unwrap();

    let req = TestRequest::default()
        .uri("/post")
        .method(Method::POST)
        .set_json(json!({
            "privacy": "public",
            "repost_of_id": &setup.other_follower_post_id
        }))
        .attach_token(&setup.token)
        .to_request();

    let resp = call_service(&app, req).await;
    assert_eq!(resp.status(), 404);
}

#[actix_web::test]
async fn post_repost_of_private_public() {
    let app = init_app().await;
    let setup = post_repost_setup(&app).await.unwrap();

    let req = TestRequest::default()
        .uri("/post")
        .method(Method::POST)
        .set_json(json!({
            "privacy": "public",
            "repost_of_id": &setup.other_private_post_id
        }))
        .attach_token(&setup.token)
        .to_request();

    let resp = call_service(&app, req).await;
    assert_eq!(resp.status(), 404);
}

#[actix_web::test]
async fn post_repost_of_my_public_public() {
    let app = init_app().await;
    let setup = post_repost_setup(&app).await.unwrap();

    let req = TestRequest::default()
        .uri("/post")
        .method(Method::POST)
        .set_json(json!({
            "privacy": "public",
            "repost_of_id": &setup.my_public_post_id
        }))
        .attach_token(&setup.token)
        .to_request();

    let resp = call_service(&app, req).await;
    assert_eq!(resp.status(), 200);
}

#[actix_web::test]
async fn post_repost_of_my_follower_public() {
    let app = init_app().await;
    let setup = post_repost_setup(&app).await.unwrap();

    let req = TestRequest::default()
        .uri("/post")
        .method(Method::POST)
        .set_json(json!({
            "privacy": "public",
            "repost_of_id": &setup.my_follower_post_id
        }))
        .attach_token(&setup.token)
        .to_request();

    let resp = call_service(&app, req).await;
    assert_eq!(resp.status(), 404);
}

#[actix_web::test]
async fn post_repost_of_my_private_public() {
    let app = init_app().await;
    let setup = post_repost_setup(&app).await.unwrap();

    let req = TestRequest::default()
        .uri("/post")
        .method(Method::POST)
        .set_json(json!({
            "privacy": "public",
            "repost_of_id": &setup.my_private_post_id
        }))
        .attach_token(&setup.token)
        .to_request();

    let resp = call_service(&app, req).await;
    assert_eq!(resp.status(), 404);
}

#[actix_web::test]
async fn post_repost_of_my_public_unlisted() {
    let app = init_app().await;
    let setup = post_repost_setup(&app).await.unwrap();

    let req = TestRequest::default()
        .uri("/post")
        .method(Method::POST)
        .set_json(json!({
            "privacy": "unlisted",
            "repost_of_id": &setup.my_public_post_id
        }))
        .attach_token(&setup.token)
        .to_request();

    let resp = call_service(&app, req).await;
    assert_eq!(resp.status(), 200);
}

#[actix_web::test]
async fn post_repost_of_my_public_follower() {
    let app = init_app().await;
    let setup = post_repost_setup(&app).await.unwrap();

    let req = TestRequest::default()
        .uri("/post")
        .method(Method::POST)
        .set_json(json!({
            "privacy": "follower",
            "repost_of_id": &setup.my_public_post_id
        }))
        .attach_token(&setup.token)
        .to_request();

    let resp = call_service(&app, req).await;
    assert_eq!(resp.status(), 400);
}

#[actix_web::test]
async fn post_repost_of_my_public_private() {
    let app = init_app().await;
    let setup = post_repost_setup(&app).await.unwrap();

    let req = TestRequest::default()
        .uri("/post")
        .method(Method::POST)
        .set_json(json!({
            "privacy": "private",
            "repost_of_id": &setup.my_public_post_id
        }))
        .attach_token(&setup.token)
        .to_request();

    let resp = call_service(&app, req).await;
    assert_eq!(resp.status(), 400);
}

struct PostViewSetup {
    token: String,
    other_follower_post_id: String,
    other_private_post_id: String,
    my_public_post_id: String,
    my_follower_post_id: String,
    my_private_post_id: String,
}

async fn post_view_setup(
    app: impl Service<Request, Response = ServiceResponse<impl MessageBody + Debug>, Error = Error>,
) -> Result<PostViewSetup, anyhow::Error> {
    register_user(&app, "testuser", "testuser", GOOD_PASSWORD).await;
    register_user(&app, "testuser2", "testuser2", GOOD_PASSWORD).await;
    let login_resp = login_user(&app, "testuser", GOOD_PASSWORD).await.unwrap();
    let login_resp2 = login_user(&app, "testuser2", GOOD_PASSWORD).await.unwrap();

    let get_post_id = {
        let app_ref = &app;
        move |token: &str, content: &str, privacy: &str| {
            let req_body = json!({
                "content": content,
                "privacy": privacy
            });
            let token = token.to_string();
            async move {
                let req = TestRequest::default()
                    .uri("/post")
                    .method(Method::POST)
                    .attach_token(&token)
                    .set_json(req_body)
                    .to_request();
                let resp = call_service(app_ref, req).await;
                assert_eq!(resp.status(), 200);
                let body: PostCreateResponse = parse_body(resp.into_body()).await.unwrap();
                body.post_id.clone()
            }
        }
    };

    let my_public_post_id = get_post_id(&login_resp.token, "public post", "public").await;
    let my_follower_post_id = get_post_id(&login_resp.token, "follower post", "follower").await;
    let my_private_post_id = get_post_id(&login_resp.token, "private post", "private").await;

    // let other_public_post_id =
    // get_post_id(&login_resp2.token, "public parent post", "public").await;
    let other_follower_post_id =
        get_post_id(&login_resp2.token, "other's follower post", "follower").await;
    let other_private_post_id =
        get_post_id(&login_resp2.token, "other's private post", "private").await;

    Ok(PostViewSetup {
        token: login_resp.token,
        my_public_post_id,
        my_follower_post_id,
        my_private_post_id,
        other_follower_post_id,
        other_private_post_id,
    })
}

#[derive(Debug, Deserialize)]
struct PostViewResponse {
    id: String,
    content: String,
}

#[actix_web::test]
async fn post_view_public() {
    let app = init_app().await;
    let setup = post_view_setup(&app).await.unwrap();

    let req = TestRequest::default()
        .uri(&format!("/post/{}", setup.my_public_post_id))
        .method(Method::GET)
        .attach_token(&setup.token)
        .to_request();

    let resp = call_service(&app, req).await;
    assert_eq!(resp.status(), 200);
    let body: PostViewResponse = parse_body(resp.into_body()).await.unwrap();
    assert_eq!(body.id, setup.my_public_post_id);
    assert_eq!(body.content, "public post");
}

#[actix_web::test]
async fn post_view_follower() {
    let app = init_app().await;
    let setup = post_view_setup(&app).await.unwrap();

    let req = TestRequest::default()
        .uri(&format!("/post/{}", setup.my_follower_post_id))
        .method(Method::GET)
        .attach_token(&setup.token)
        .to_request();

    let resp = call_service(&app, req).await;
    assert_eq!(resp.status(), 200);
    let body: PostViewResponse = parse_body(resp.into_body()).await.unwrap();
    assert_eq!(body.id, setup.my_follower_post_id);
    assert_eq!(body.content, "follower post");
}

#[actix_web::test]
async fn post_view_private() {
    let app = init_app().await;
    let setup = post_view_setup(&app).await.unwrap();

    let req = TestRequest::default()
        .uri(&format!("/post/{}", setup.my_private_post_id))
        .method(Method::GET)
        .attach_token(&setup.token)
        .to_request();

    let resp = call_service(&app, req).await;
    assert_eq!(resp.status(), 200);
    let body: PostViewResponse = parse_body(resp.into_body()).await.unwrap();
    assert_eq!(body.id, setup.my_private_post_id);
    assert_eq!(body.content, "private post");
}

#[actix_web::test]
async fn post_view_others_follower() {
    let app = init_app().await;
    let setup = post_view_setup(&app).await.unwrap();

    let req = TestRequest::default()
        .uri(&format!("/post/{}", setup.other_follower_post_id))
        .method(Method::GET)
        .attach_token(&setup.token)
        .to_request();

    let resp = call_service(&app, req).await;
    assert_eq!(resp.status(), 404);
}

#[actix_web::test]
async fn post_view_others_private() {
    let app = init_app().await;
    let setup = post_view_setup(&app).await.unwrap();

    let req = TestRequest::default()
        .uri(&format!("/post/{}", setup.other_private_post_id))
        .method(Method::GET)
        .attach_token(&setup.token)
        .to_request();

    let resp = call_service(&app, req).await;
    assert_eq!(resp.status(), 404);
}

struct FollowSetup {
    token1: String,
    token2: String,
}

async fn setup_follow(
    app: impl Service<Request, Response = ServiceResponse<impl MessageBody + Debug>, Error = Error>,
) -> FollowSetup {
    register_user(&app, "user1", "testuser", GOOD_PASSWORD).await;
    register_user(&app, "user2", "testuser2", GOOD_PASSWORD).await;
    let login_resp = login_user(&app, "user1", GOOD_PASSWORD).await.unwrap();
    let login_resp2 = login_user(&app, "user2", GOOD_PASSWORD).await.unwrap();
    FollowSetup {
        token1: login_resp.token,
        token2: login_resp2.token,
    }
}

async fn follow_or_unfollow(
    app: impl Service<Request, Response = ServiceResponse<impl MessageBody + Debug>, Error = Error>,
    token: &str,
    user: &str,
    follow: bool,
) -> ServiceResponse<impl MessageBody + Debug> {
    let req = TestRequest::default()
        .uri(&format!("/user/{}/follow", user))
        .method(if follow { Method::PUT } else { Method::DELETE })
        .attach_token(token)
        .to_request();
    let resp = call_service(&app, req).await;
    resp
}

#[actix_web::test]
async fn follow_user() {
    let app = init_app().await;
    let setup = setup_follow(&app).await;

    let resp = follow_or_unfollow(&app, &setup.token1, "@user2", true).await;
    assert_eq!(resp.status(), 200);
}

#[actix_web::test]
async fn unfollow_user() {
    let app = init_app().await;
    let setup = setup_follow(&app).await;

    let resp = follow_or_unfollow(&app, &setup.token1, "@user2", true).await;
    assert_eq!(resp.status(), 200);

    let resp = follow_or_unfollow(&app, &setup.token1, "@user2", false).await;
    assert_eq!(resp.status(), 200);
}

#[derive(Debug, Deserialize)]
struct ListResponse<T> {
    result: Vec<T>,
}

#[derive(Debug, Deserialize)]
struct UserResponse {
    username: String,
}

#[actix_web::test]
async fn get_followers() {
    let app = init_app().await;
    let setup = setup_follow(&app).await;

    let resp = follow_or_unfollow(&app, &setup.token1, "@user2", true).await;
    assert_eq!(resp.status(), 200);

    let req = TestRequest::default()
        .uri("/user/@user2/followers")
        .attach_token(&setup.token2)
        .to_request();
    let resp = call_service(&app, req).await;

    assert_eq!(resp.status(), 200);
    let body: ListResponse<UserResponse> = parse_body(resp.into_body()).await.unwrap();
    assert_eq!(body.result.len(), 1);
    assert_eq!(body.result[0].username, "user1");
}

#[actix_web::test]
async fn get_followings() {
    let app = init_app().await;
    let setup = setup_follow(&app).await;

    let resp = follow_or_unfollow(&app, &setup.token1, "@user2", true).await;
    assert_eq!(resp.status(), 200);

    let req = TestRequest::default()
        .uri("/user/@user1/following")
        .attach_token(&setup.token1)
        .to_request();
    let resp = call_service(&app, req).await;

    assert_eq!(resp.status(), 200);
    let body: ListResponse<UserResponse> = parse_body(resp.into_body()).await.unwrap();
    assert_eq!(body.result.len(), 1);
    assert_eq!(body.result[0].username, "user2");
}

struct PostListSetup {
    token1: String,
    token2: String,
    token3: String,
    token4: String,
}

async fn post_list_setup(
    app: impl Service<Request, Response = ServiceResponse<impl MessageBody + Debug>, Error = Error>,
) -> Result<PostListSetup, anyhow::Error> {
    register_user(&app, "user1", "user1", GOOD_PASSWORD).await;
    register_user(&app, "user2", "user2", GOOD_PASSWORD).await;
    register_user(&app, "user3", "user3", GOOD_PASSWORD).await;
    register_user(&app, "user4", "user4", GOOD_PASSWORD).await;
    let login_resp = login_user(&app, "user1", GOOD_PASSWORD).await.unwrap();
    let login_resp2 = login_user(&app, "user2", GOOD_PASSWORD).await.unwrap();
    let login_resp3 = login_user(&app, "user3", GOOD_PASSWORD).await.unwrap();
    let login_resp4 = login_user(&app, "user4", GOOD_PASSWORD).await.unwrap();

    let get_post_id = {
        let app_ref = &app;
        move |token: &str, content: &str, privacy: &str| {
            let req_body = json!({
                "content": content,
                "privacy": privacy
            });
            let token = token.to_string();
            async move {
                let req = TestRequest::default()
                    .uri("/post")
                    .method(Method::POST)
                    .attach_token(&token)
                    .set_json(req_body)
                    .to_request();
                let resp = call_service(app_ref, req).await;
                assert_eq!(resp.status(), 200);
                let body: PostCreateResponse = parse_body(resp.into_body()).await.unwrap();
                body.post_id.clone()
            }
        }
    };

    // user3 follows user1
    let resp = follow_or_unfollow(&app, &login_resp3.token, "@user1", true).await;
    assert_eq!(resp.status(), 200);

    let _ = get_post_id(&login_resp.token, "public content", "public").await;
    let _ = get_post_id(&login_resp.token, "unlisted content", "unlisted").await;
    let _ = get_post_id(&login_resp.token, "follower content", "follower").await;
    let _ = get_post_id(&login_resp.token, "private content @user4", "private").await;

    Ok(PostListSetup {
        token1: login_resp.token,
        token2: login_resp2.token,
        token3: login_resp3.token,
        token4: login_resp4.token,
    })
}

#[derive(Debug, Deserialize)]
struct PostResponse {
    content: String,
}

#[actix_web::test]
async fn post_list_my_posts() {
    let app = init_app().await;
    let setup = post_list_setup(&app).await.unwrap();

    let req = TestRequest::default()
        .uri("/user/@user1/posts")
        .attach_token(&setup.token1)
        .to_request();
    let resp = call_service(&app, req).await;
    assert_eq!(resp.status(), 200);

    let body: ListResponse<PostResponse> = parse_body(resp.into_body()).await.unwrap();
    assert_eq!(body.result.len(), 4);
    assert_eq!(body.result[0].content, "private content @user4");
    assert_eq!(body.result[1].content, "follower content");
    assert_eq!(body.result[2].content, "unlisted content");
    assert_eq!(body.result[3].content, "public content");
}

#[actix_web::test]
async fn post_list_others_posts() {
    let app = init_app().await;
    let setup = post_list_setup(&app).await.unwrap();

    let req = TestRequest::default()
        .uri("/user/@user1/posts")
        .attach_token(&setup.token2)
        .to_request();
    let resp = call_service(&app, req).await;
    assert_eq!(resp.status(), 200);

    let body: ListResponse<PostResponse> = parse_body(resp.into_body()).await.unwrap();
    assert_eq!(body.result.len(), 2);
    assert_eq!(body.result[0].content, "unlisted content");
    assert_eq!(body.result[1].content, "public content");
}

#[actix_web::test]
async fn post_list_followers_posts() {
    let app = init_app().await;
    let setup = post_list_setup(&app).await.unwrap();

    let req = TestRequest::default()
        .uri("/user/@user1/posts")
        .attach_token(&setup.token3)
        .to_request();
    let resp = call_service(&app, req).await;
    assert_eq!(resp.status(), 200);

    let body: ListResponse<PostResponse> = parse_body(resp.into_body()).await.unwrap();
    assert_eq!(body.result.len(), 3);
    assert_eq!(body.result[0].content, "follower content");
    assert_eq!(body.result[1].content, "unlisted content");
    assert_eq!(body.result[2].content, "public content");
}

#[actix_web::test]
async fn post_list_mentioned_posts() {
    let app = init_app().await;
    let setup = post_list_setup(&app).await.unwrap();

    let req = TestRequest::default()
        .uri("/user/@user1/posts")
        .attach_token(&setup.token4)
        .to_request();
    let resp = call_service(&app, req).await;
    assert_eq!(resp.status(), 200);

    let body: ListResponse<PostResponse> = parse_body(resp.into_body()).await.unwrap();
    assert_eq!(body.result.len(), 3);
    assert_eq!(body.result[0].content, "private content @user4");
    assert_eq!(body.result[1].content, "unlisted content");
    assert_eq!(body.result[2].content, "public content");
}

struct TimelineSetup {
    token1: String,
    token2: String,
    token3: String,
    token4: String,
}

async fn timeline_setup(
    app: impl Service<Request, Response = ServiceResponse<impl MessageBody + Debug>, Error = Error>,
) -> Result<TimelineSetup, anyhow::Error> {
    register_user(&app, "user1", "user1", GOOD_PASSWORD).await;
    register_user(&app, "user2", "user2", GOOD_PASSWORD).await;
    register_user(&app, "user3", "user3", GOOD_PASSWORD).await;
    register_user(&app, "user4", "user4", GOOD_PASSWORD).await;
    let login_resp = login_user(&app, "user1", GOOD_PASSWORD).await.unwrap();
    let login_resp2 = login_user(&app, "user2", GOOD_PASSWORD).await.unwrap();
    let login_resp3 = login_user(&app, "user3", GOOD_PASSWORD).await.unwrap();
    let login_resp4 = login_user(&app, "user4", GOOD_PASSWORD).await.unwrap();

    let get_post_id = {
        let app_ref = &app;
        move |token: &str, content: &str, privacy: &str| {
            let req_body = json!({
                "content": content,
                "privacy": privacy
            });
            let token = token.to_string();
            async move {
                let req = TestRequest::default()
                    .uri("/post")
                    .method(Method::POST)
                    .attach_token(&token)
                    .set_json(req_body)
                    .to_request();
                let resp = call_service(app_ref, req).await;
                assert_eq!(resp.status(), 200);
                let body: PostCreateResponse = parse_body(resp.into_body()).await.unwrap();
                body.post_id.clone()
            }
        }
    };

    // user3 follows user1
    let resp = follow_or_unfollow(&app, &login_resp3.token, "@user1", true).await;
    assert_eq!(resp.status(), 200);

    let _ = get_post_id(&login_resp.token, "public content", "public").await;
    let _ = get_post_id(&login_resp.token, "unlisted content", "unlisted").await;
    let _ = get_post_id(&login_resp.token, "follower content", "follower").await;
    let _ = get_post_id(&login_resp.token, "private content @user4", "private").await;

    Ok(TimelineSetup {
        token1: login_resp.token,
        token2: login_resp2.token,
        token3: login_resp3.token,
        token4: login_resp4.token,
    })
}

#[actix_web::test]
async fn timeline_my_posts() {
    let app = init_app().await;
    let setup = timeline_setup(&app).await.unwrap();

    let req = TestRequest::default()
        .uri("/timeline")
        .attach_token(&setup.token1)
        .to_request();
    let resp = call_service(&app, req).await;
    assert_eq!(resp.status(), 200);

    let body: ListResponse<PostResponse> = parse_body(resp.into_body()).await.unwrap();
    assert_eq!(body.result.len(), 4);
    assert_eq!(body.result[0].content, "private content @user4");
    assert_eq!(body.result[1].content, "follower content");
    assert_eq!(body.result[2].content, "unlisted content");
    assert_eq!(body.result[3].content, "public content");
}

#[actix_web::test]
async fn timeline_empty() {
    let app = init_app().await;
    let setup = timeline_setup(&app).await.unwrap();

    let req = TestRequest::default()
        .uri("/timeline")
        .attach_token(&setup.token2)
        .to_request();
    let resp = call_service(&app, req).await;
    assert_eq!(resp.status(), 200);

    let body: ListResponse<PostResponse> = parse_body(resp.into_body()).await.unwrap();
    assert_eq!(body.result.len(), 0);
}

#[actix_web::test]
async fn timeline_follows() {
    let app = init_app().await;
    let setup = timeline_setup(&app).await.unwrap();

    let req = TestRequest::default()
        .uri("/timeline")
        .attach_token(&setup.token3)
        .to_request();
    let resp = call_service(&app, req).await;
    assert_eq!(resp.status(), 200);

    let body: ListResponse<PostResponse> = parse_body(resp.into_body()).await.unwrap();
    assert_eq!(body.result.len(), 3);
    assert_eq!(body.result[0].content, "follower content");
    assert_eq!(body.result[1].content, "unlisted content");
    assert_eq!(body.result[2].content, "public content");
}

#[actix_web::test]
async fn timeline_mentioned() {
    let app = init_app().await;
    let setup = timeline_setup(&app).await.unwrap();

    let req = TestRequest::default()
        .uri("/timeline")
        .attach_token(&setup.token4)
        .to_request();
    let resp = call_service(&app, req).await;
    assert_eq!(resp.status(), 200);

    let body: ListResponse<PostResponse> = parse_body(resp.into_body()).await.unwrap();
    assert_eq!(body.result.len(), 1);
    assert_eq!(body.result[0].content, "private content @user4");
}

struct ReactionSetup {
    token1: String,
    token2: String,
    public_post_id: String,
}

async fn reaction_setup(
    app: impl Service<Request, Response = ServiceResponse<impl MessageBody + Debug>, Error = Error>,
) -> ReactionSetup {
    register_user(&app, "user1", "user1", GOOD_PASSWORD).await;
    register_user(&app, "user2", "user2", GOOD_PASSWORD).await;
    let login_resp = login_user(&app, "user1", GOOD_PASSWORD).await.unwrap();
    let login_resp2 = login_user(&app, "user2", GOOD_PASSWORD).await.unwrap();

    let req = TestRequest::default()
        .uri("/post")
        .method(Method::POST)
        .attach_token(&login_resp.token)
        .set_json(json!({
            "content": "public content",
            "privacy": "public"
        }))
        .to_request();
    let resp = call_service(&app, req).await;
    assert_eq!(resp.status(), 200);
    let body: PostCreateResponse = parse_body(resp.into_body()).await.unwrap();

    ReactionSetup {
        token1: login_resp.token,
        token2: login_resp2.token,
        public_post_id: body.post_id,
    }
}

#[actix_web::test]
async fn favorite_public() {
    let app = init_app().await;
    let setup = reaction_setup(&app).await;

    let req = TestRequest::default()
        .uri(&format!("/post/{}/favorite", setup.public_post_id))
        .method(Method::PUT)
        .attach_token(&setup.token2)
        .to_request();
    let resp = call_service(&app, req).await;
    assert_eq!(resp.status(), 200);
}

#[actix_web::test]
async fn bookmark_public() {
    let app = init_app().await;
    let setup = reaction_setup(&app).await;

    let req = TestRequest::default()
        .uri(&format!("/post/{}/bookmark", setup.public_post_id))
        .method(Method::PUT)
        .attach_token(&setup.token2)
        .to_request();
    let resp = call_service(&app, req).await;
    assert_eq!(resp.status(), 200);
}

#[actix_web::test]
async fn reaction_public() {
    let app = init_app().await;
    let setup = reaction_setup(&app).await;

    let req = TestRequest::default()
        .uri(&format!("/post/{}/reaction", setup.public_post_id))
        .method(Method::POST)
        .set_json(json!( {
            "reaction": "🎉",
            "add": true
        }))
        .attach_token(&setup.token2)
        .to_request();
    let resp = call_service(&app, req).await;
    assert_eq!(resp.status(), 200);
}

#[actix_web::test]
async fn favorite_delete_public() {
    let app = init_app().await;
    let setup = reaction_setup(&app).await;

    let req = TestRequest::default()
        .uri(&format!("/post/{}/favorite", setup.public_post_id))
        .method(Method::PUT)
        .attach_token(&setup.token2)
        .to_request();
    let resp = call_service(&app, req).await;
    assert_eq!(resp.status(), 200);

    let req = TestRequest::default()
        .uri(&format!("/post/{}/favorite", setup.public_post_id))
        .method(Method::DELETE)
        .attach_token(&setup.token2)
        .to_request();
    let resp = call_service(&app, req).await;
    assert_eq!(resp.status(), 200);
}

#[actix_web::test]
async fn bookmark_delete_public() {
    let app = init_app().await;
    let setup = reaction_setup(&app).await;

    let req = TestRequest::default()
        .uri(&format!("/post/{}/bookmark", setup.public_post_id))
        .method(Method::PUT)
        .attach_token(&setup.token2)
        .to_request();
    let resp = call_service(&app, req).await;
    assert_eq!(resp.status(), 200);

    let req = TestRequest::default()
        .uri(&format!("/post/{}/bookmark", setup.public_post_id))
        .method(Method::DELETE)
        .attach_token(&setup.token2)
        .to_request();
    let resp = call_service(&app, req).await;
    assert_eq!(resp.status(), 200);
}

#[actix_web::test]
async fn reaction_delete_public() {
    let app = init_app().await;
    let setup = reaction_setup(&app).await;

    let req = TestRequest::default()
        .uri(&format!("/post/{}/reaction", setup.public_post_id))
        .method(Method::POST)
        .set_json(json!( {
            "reaction": "🎉",
            "add": true
        }))
        .attach_token(&setup.token2)
        .to_request();
    let resp = call_service(&app, req).await;
    assert_eq!(resp.status(), 200);

    let req = TestRequest::default()
        .uri(&format!("/post/{}/reaction", setup.public_post_id))
        .method(Method::POST)
        .set_json(json!( {
            "reaction": "🎉",
            "add": false
        }))
        .attach_token(&setup.token2)
        .to_request();
    let resp = call_service(&app, req).await;
    assert_eq!(resp.status(), 200);
}
