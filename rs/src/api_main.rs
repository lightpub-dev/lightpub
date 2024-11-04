use actix_cors::Cors;
use actix_web::http::header;
use actix_web::{middleware::Logger, web, App, HttpServer};
use clap::Parser;
use lightpub::api::state::AppState;
use lightpub::api_root::{
    add_post_bookmark, add_post_favorite, delete_post_bookmark, delete_post_favorite,
    delete_single_post, file_upload, get_single_post, get_user_followers, get_user_following,
    get_user_outbox, get_user_posts, host_meta, login, logout_user, modify_post_reaction,
    node_info_2_0, node_info_2_1, post_post, register, timeline, truncate_database,
    update_my_profile, user_create_follow, user_delete_follow, user_get, user_inbox, webfinger,
    well_known_node_info,
};
use lightpub::config::Config;
use sqlx::sqlite::SqlitePoolOptions;
use std::io::Read;
use std::path::PathBuf;

#[derive(Parser)]
#[command(version, about, long_about = None)]
struct Cli {
    /// Sets a custom config file
    #[arg(short, long, value_name = "FILE")]
    config: PathBuf,
}

#[actix_web::main]
async fn main() -> std::io::Result<()> {
    tracing_subscriber::fmt()
        .with_max_level(tracing::Level::DEBUG)
        .with_test_writer()
        .init();

    let cli = Cli::parse();
    let config = cli.config.as_path();

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

    let app_state = AppState::new(pool, config.clone());

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
            .service(logout_user)
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
            .service(modify_post_reaction);

        if config.dev.debug {
            app = app.service(truncate_database);
        }

        app
    })
    .bind(("0.0.0.0", 8000))?
    .run()
    .await
}
