/*
Lightpub: a simple ActivityPub server
Copyright (C) 2025 tinaxd

This program is free software: you can redistribute it and/or modify
it under the terms of the GNU Affero General Public License as published
by the Free Software Foundation, either version 3 of the License, or
(at your option) any later version.

This program is distributed in the hope that it will be useful,
but WITHOUT ANY WARRANTY; without even the implied warranty of
MERCHANTABILITY or FITNESS FOR A PARTICULAR PURPOSE.  See the
GNU Affero General Public License for more details.

You should have received a copy of the GNU Affero General Public License
along with this program.  If not, see <https://www.gnu.org/licenses/>.
*/

use std::{fs::File, path::PathBuf, str::FromStr, sync::Arc, time::Duration};

use activitypub_federation::config::FederationConfig;
use actix_multipart::form::tempfile::TempFileConfig;
use actix_session::{storage::RedisSessionStore, SessionMiddleware};
use actix_web::{
    cookie::Key,
    middleware::{Compress, Logger, NormalizePath},
    web, App, HttpServer,
};
use lightpub_rs::{
    api::{
        self,
        admin::admin_api_rebuild_note_fulltext,
        auth::{api_change_password, api_login_user, api_logout_user, api_register_user},
        note::{
            api_create_note, api_create_renote, api_edit_note_view, api_get_note,
            api_get_note_like_users, api_get_note_mentions_users, api_get_note_renote_users,
            api_note_add_bookmark, api_note_add_like, api_note_delete,
            api_note_delete_by_renote_target_id, api_note_patch, api_note_remove_bookmark,
            api_note_remove_like,
        },
        notifications::{
            api_get_notifications, api_read_all_notifications, api_read_notification,
            api_unread_notification_count, api_wp_public_key, api_wp_subscribe,
        },
        search::api_search,
        serve_sw_js,
        timeline::api_get_timeline,
        trends::api_get_trends,
        upload::api_get_upload,
        user::{
            api_get_user, api_get_user_avatar, api_get_user_notes, api_user_followers_list,
            api_user_followings_list, api_user_inbox, api_user_interaction, api_user_outbox,
            api_user_profile_update,
        },
    },
    client::{
        note::{
            client_get_note, client_note_liked_list, client_note_mentions_list,
            client_note_renotes_list,
        },
        notification::client_notification_get,
        search::client_get_search,
        timeline::client_timeline,
        user::{
            client_change_password, client_edit_profile_get, client_get_profile, client_login_user,
            client_my_profile, client_register_user, client_user_followers_list,
            client_user_followings_list,
        },
    },
    create_handlebars, create_http_client, create_http_client_with_cache, registeration_open,
    AppConfigBuilder, AppState, NodeInfoBuilder,
};
use lightpub_service::{
    services::{
        db::{Conn, RedisConn},
        fulltext::FTClient,
        notification::push::WPClient,
        queue::{ApubWorker, QConn},
    },
    ServiceState, ServiceStateBase,
};
use migration::{Migrator, MigratorTrait};
use sea_orm::ConnectOptions;
use tokio_util::sync::CancellationToken;
use tracing::info;
use url::Url;
use web_push::{IsahcWebPushClient, VapidSignatureBuilder};

const TYPESENSE_DEFAULT_LOCALE: &str = "ja";

fn get_tmp_dirs() -> TmpDirConfig {
    let mut cfg = TmpDirConfig::default();
    if let Ok(tmp_dir) = std::env::var("TMP_DIR") {
        let base_path = PathBuf::from_str(&tmp_dir).expect("TMP_DIR should be a valid path");
        if !base_path.is_dir() {
            panic!("TMP_DIR should be a directory");
        }
        cfg.upload_tmp = base_path.join("uploads").into();
        cfg.proxy_cache = base_path.join("proxy_cache").into();
    }

    if let Some(tmp) = cfg.upload_tmp.as_ref() {
        std::fs::create_dir_all(tmp).expect("failed to create upload tmp dir");
    }
    if let Some(tmp) = cfg.proxy_cache.as_ref() {
        std::fs::create_dir_all(tmp).expect("failed to create proxy cache dir");
    }

    cfg
}

#[derive(Debug, Default)]
struct TmpDirConfig {
    upload_tmp: Option<PathBuf>,
    proxy_cache: Option<PathBuf>,
}

#[tokio::main]
async fn main() {
    match dotenvy::dotenv() {
        Ok(_) => {}
        Err(e) => info!("failed to load .env: {:?}", e),
    }

    let dev_mode = std::env::var("DEV_MODE").is_ok_and(|x| x == "true");

    // initialize tracing
    // console_subscriber::init();
    tracing_subscriber::fmt::init();
    // env_logger::init();

    if dev_mode {
        info!("Running in dev mode");
    }

    // database
    let database_url = std::env::var("DATABASE_URL").expect("DATABASE_URL is set");
    let mut db_options = ConnectOptions::new(&database_url);
    db_options.max_connections(
        std::env::var("DB_MAX_CONNECTIONS")
            .map(|n| n.parse::<u32>().expect("DB_MAX_CONNECTIONS is a number"))
            .unwrap_or(10),
    );
    db_options.idle_timeout(Duration::from_secs(
        std::env::var("DB_IDLE_TIMEOUT")
            .map(|n| n.parse::<u64>().expect("DB_IDLE_TIMEOUT is a number"))
            .unwrap_or(30),
    ));
    let conn = Conn::create(db_options).await;

    // redis
    let session_key = Key::from(
        std::env::var("SESSION_KEY")
            .expect("SESSION_KEY is set")
            .as_bytes(),
    );
    let redis_url = std::env::var("REDIS_URL").expect("REDIS_URL is set");
    let redis_store = RedisSessionStore::new(redis_url.clone()).await.unwrap();

    let rconn = RedisConn::new(
        redis::aio::ConnectionManager::new(
            redis::Client::open(redis_url).expect("failed to create redis client"),
        )
        .await
        .expect("failed to create redis connection"),
    );

    // nats
    let nats_url = std::env::var("NATS_URL").expect("NATS_URL is set");
    let nats_conn = QConn::connect(nats_url)
        .await
        .expect("failed to connect to nats");

    let num_workers = std::env::var("NUM_WORKERS")
        .map(|n| n.parse::<usize>().expect("NUM_WORKERS is a number"))
        .ok();

    Migrator::up(conn.db(), None)
        .await
        .expect("migration failed");

    // typesense
    let typesense_url = std::env::var("TYPESENSE_URL");
    let ft_client = match typesense_url {
        Ok(url) if url.trim().is_empty() => {
            info!("TYPESENSE_URL is empty, fulltext search will be disabled");
            None
        }
        Ok(url) => {
            let client = reqwest_middleware::ClientBuilder::new(reqwest::Client::default()).build();
            let typesense_api_key =
                std::env::var("TYPESENSE_API_KEY").expect("TYPESENSE_API_KEY is set");
            let typesense_locale = std::env::var("TYPESENSE_LOCALE")
                .unwrap_or_else(|_| TYPESENSE_DEFAULT_LOCALE.to_string());
            info!(
                "Fulltext search enabled, using Typesense at {}, locale={}",
                url, typesense_locale
            );

            Some(FTClient::new(
                client,
                typesense_api_key,
                typesense_locale,
                url.parse().expect("TYPESENSE_URL should be a valid URL"),
            ))
        }
        Err(_) => {
            info!("TYPESENSE_URL is not set, fulltext search will be disabled");
            None
        }
    };

    // Webpush
    let webpush_key = std::env::var("WEBPUSH_VAPID_KEY");
    let wp_client = match webpush_key {
        Ok(k) if k.trim().is_empty() => {
            info!("WEBPUSH_VAPID_KEY is empty, webpush notifications will be disabled");
            None
        }
        Err(_) => {
            info!("WEBPUSH_VAPID_KEY is not set, webpush notifications will be disabled");
            None
        }
        Ok(k) => {
            let private_key_file = File::open(k).expect("failed to open WEBPUSH_VAPID_KEY file");
            Some(WPClient::new(
                IsahcWebPushClient::new().expect("failed to create webpush client"),
                VapidSignatureBuilder::from_pem_no_sub(private_key_file)
                    .expect("failed to create vapid signature builder"),
            ))
        }
    };

    // templates
    let handlebars = create_handlebars(dev_mode);

    // my domain
    let base_url = {
        let env = std::env::var("LP_BASE_URL").ok();
        match (env, dev_mode) {
            (Some(base_url), _) => {
                info!("Base URL is set to {}", base_url);
                Url::from_str(&base_url).expect("LP_BASE_URL should be a valid URL")
            }
            (None, true) => {
                info!("Running in dev mode, using example.com as base URL");
                "https://example.com".parse().unwrap()
            }
            (None, false) => panic!("LP_BASE_URL is not set"),
        }
    };

    // tmp dirs
    let tmp_dirs = get_tmp_dirs();

    // client
    let client = create_http_client_with_cache(dev_mode, &base_url, tmp_dirs.proxy_cache.clone());

    // node info
    let node_name = std::env::var("NODE_NAME").ok();
    let node_description = std::env::var("NODE_DESCRIPTION").ok();
    let mut nodeinfo = NodeInfoBuilder::default();
    if let Some(name) = node_name {
        nodeinfo.name(name);
    }
    if let Some(desc) = node_description {
        nodeinfo.description(desc);
    }
    let nodeinfo = nodeinfo.build().unwrap();

    // app state
    let base_state = ServiceStateBase::new(
        conn.clone(),
        Arc::new(rconn),
        // Arc::new(DummyKV::default()),
        nats_conn.clone(),
        dev_mode,
        base_url.clone(),
        client,
        ft_client,
        wp_client,
    );

    // activitypub federation config
    let config = FederationConfig::builder()
        .domain(base_url.domain().expect("base_url should have a domain"))
        .app_data(base_state.clone())
        .client(create_http_client(dev_mode))
        .debug(dev_mode)
        .build()
        .await
        .expect("failed to build federation config");

    // open registration
    let open_registration = registeration_open();

    // apub error reports
    let report_apub_parse_errors =
        std::env::var("REPORT_APUB_PARSE_ERRORS").is_ok_and(|x| x == "true");

    let state = ServiceState::new(base_state, config);
    let config = AppConfigBuilder::default()
        .nodeinfo(nodeinfo)
        .open_registration(open_registration)
        .report_apub_parse_errors(report_apub_parse_errors)
        .build()
        .unwrap();
    let state = AppState::new(state, handlebars, config);

    lightpub_service::services::init_service(state.service_state())
        .await
        .expect("failed to initialize lightpub services");

    // Run apub worker
    let worker = ApubWorker::new(nats_conn.clone());
    let worker_cancel = CancellationToken::new();
    let worker_handle = tokio::spawn({
        let worker_cancel = worker_cancel.clone();
        let fed_data = state.request_data();
        async move {
            worker.start(&fed_data, &worker_cancel).await.unwrap();
        }
    });

    // run our app with hyper, listening globally on port 3000
    let mut server = HttpServer::new(move || {
        // tempfile config
        let mut tempfile_config = TempFileConfig::default();
        if let Some(tmp_dir) = tmp_dirs.upload_tmp.as_ref() {
            tempfile_config = tempfile_config.directory(tmp_dir);
        }

        App::new()
            .service(web::redirect("/", "/client/timeline"))
            .service(admin_api_rebuild_note_fulltext)
            .service(
                web::scope("/auth")
                    .service(api_register_user)
                    .service(api_login_user)
                    .service(api_logout_user)
                    .service(api_change_password),
            )
            .service(
                web::scope("/note")
                    .service(api_create_note)
                    .service(api_get_note)
                    .service(api_note_delete)
                    .service(api_note_delete_by_renote_target_id)
                    .service(api_note_patch)
                    .service(api_edit_note_view)
                    .service(api_note_add_like)
                    .service(api_note_remove_like)
                    .service(api_note_add_bookmark)
                    .service(api_note_remove_bookmark)
                    .service(api_create_renote)
                    .service(api_get_note_like_users)
                    .service(api_get_note_renote_users)
                    .service(api_get_note_mentions_users),
            )
            .service(
                web::scope("/user")
                    .service(api_get_user)
                    .service(api_get_user_avatar)
                    .service(api_get_user_notes)
                    .service(api_user_interaction)
                    .service(api_user_profile_update)
                    .service(api_user_followers_list)
                    .service(api_user_followings_list)
                    .service(api_user_inbox)
                    .service(api_user_outbox),
            )
            .service(api_get_timeline)
            .service(
                web::scope("/notification")
                    .service(api_get_notifications)
                    .service(api_read_all_notifications)
                    .service(api_read_notification)
                    .service(api_unread_notification_count)
                    .service(api_wp_subscribe)
                    .service(api_wp_public_key),
            )
            .service(api_search)
            .service(api_get_trends)
            .service(api_get_upload)
            .service(
                web::scope("/client")
                    .service(client_register_user)
                    .service(client_login_user)
                    .service(client_change_password)
                    .service(client_timeline)
                    .service(client_my_profile)
                    .service(client_get_profile)
                    .service(client_user_followings_list)
                    .service(client_user_followers_list)
                    .service(client_edit_profile_get)
                    .service(client_get_note)
                    .service(client_note_renotes_list)
                    .service(client_note_liked_list)
                    .service(client_note_mentions_list)
                    .service(client_notification_get)
                    .service(client_get_search),
            )
            .route("/healthcheck", web::get().to(|| async { "OK" }))
            .service(api::federation::webfinger)
            .service(api::federation::api_shared_inbox)
            .service(api::federation::nodeinfo::nodeinfo)
            .service(api::federation::nodeinfo::nodeinfo_2_1)
            .service(serve_sw_js)
            .service(actix_files::Files::new("/static", "./static"))
            .wrap(
                SessionMiddleware::builder(redis_store.clone(), session_key.clone())
                    .cookie_same_site(actix_web::cookie::SameSite::Lax)
                    .cookie_secure(!dev_mode)
                    .cookie_http_only(true)
                    .build(),
            )
            .wrap(NormalizePath::trim())
            .wrap(Compress::default())
            .wrap(Logger::default())
            .app_data(web::Data::new(state.clone()))
            .app_data(web::Data::new(tempfile_config))
    });

    if let Some(num_workers) = num_workers {
        server = server.workers(num_workers);
    }

    server.bind(("0.0.0.0", 8000)).unwrap().run().await.unwrap();

    worker_cancel.cancel();
    worker_handle.await.unwrap();
}
