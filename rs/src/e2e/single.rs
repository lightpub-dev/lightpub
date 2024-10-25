use crate::api_root::{
    add_post_bookmark, add_post_favorite, delete_post_bookmark, delete_post_favorite,
    delete_single_post, file_upload, get_single_post, get_user_followers, get_user_following,
    get_user_outbox, get_user_posts, host_meta, login, modify_post_reaction, node_info_2_0,
    node_info_2_1, post_post, register, timeline, truncate_database, update_my_profile,
    user_create_follow, user_delete_follow, user_get, user_inbox, webfinger, well_known_node_info,
};
use std::io::Read;

use actix_http::Request;
use actix_web::error::Error;
use actix_web::{
    body::MessageBody,
    dev::{Service, ServiceFactory, ServiceRequest, ServiceResponse},
    test::{call_service, init_service, TestRequest},
    web, App,
};
use reqwest::Method;
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

async fn register_admin(
    app: impl Service<Request, Response = ServiceResponse<impl MessageBody + Debug>, Error = Error>,
) {
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
