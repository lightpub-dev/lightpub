use std::sync::Arc;

use activitypub_federation::config::FederationConfig;
use migration::{Migrator, MigratorTrait};
use once_cell::sync::Lazy;
use sea_orm::ConnectOptions;
use testcontainers::{ContainerAsync, ImageExt, runners::AsyncRunner};
use testcontainers_modules::{
    mariadb::Mariadb,
    nats::{Nats, NatsServerCmd},
    redis::Redis,
};
use url::{Host, Url};

use crate::{
    ServiceState, ServiceStateBase,
    services::{
        db::{Conn, RedisConn},
        queue::QConn,
    },
};

pub struct TestState {
    #[allow(dead_code)]
    db: ContainerAsync<Mariadb>,
    #[allow(dead_code)]
    kv: ContainerAsync<Redis>,
    #[allow(dead_code)]
    nats: ContainerAsync<Nats>,
    pub app: ServiceState,
}

async fn test_with_mariadb() -> (ContainerAsync<Mariadb>, Host, u16) {
    let container = Mariadb::default()
        .with_tag("lts")
        .start()
        .await
        .expect("Failed to start Mariadb container");
    let ip = container.get_host().await.unwrap();
    let port = container.get_host_port_ipv4(3306).await.unwrap();
    (container, ip, port)
}

async fn test_with_redis() -> (ContainerAsync<Redis>, Host, u16) {
    let container = Redis::default()
        .with_tag("latest")
        .start()
        .await
        .expect("Failed to start Redis container");
    let ip = container.get_host().await.unwrap();
    let port = container.get_host_port_ipv4(6379).await.unwrap();
    (container, ip, port)
}

async fn test_with_nats() -> (ContainerAsync<Nats>, Host, u16) {
    let container = Nats::default()
        .with_cmd(&NatsServerCmd::default().with_jetstream())
        .start()
        .await
        .expect("failed to start nats container");
    let ip = container.get_host().await.unwrap();
    let port = container.get_host_port_ipv4(4222).await.unwrap();
    (container, ip, port)
}

pub async fn test_setup() -> TestState {
    let ((db, conn), (kv, kv_conn), (nats, nats_conn)) = tokio::join!(
        async {
            let (db, db_ip, db_port) = test_with_mariadb().await;
            let mut conn_opts =
                ConnectOptions::new(format!("mysql://{}:{}/{}", db_ip, db_port, "test"));
            conn_opts.max_connections(30);
            let conn = Conn::create(conn_opts).await;
            (db, conn)
        },
        async {
            let (kv, kv_ip, kv_port) = test_with_redis().await;
            let kv_conn = redis::aio::ConnectionManager::new(
                redis::Client::open(format!("redis://{}:{}", kv_ip, kv_port)).unwrap(),
            )
            .await
            .unwrap();
            (kv, kv_conn)
        },
        async {
            let (nats, nats_ip, nats_port) = test_with_nats().await;
            let nats_conn = QConn::connect(format!("nats://{nats_ip}:{nats_port}"))
                .await
                .unwrap();
            (nats, nats_conn)
        }
    );

    Migrator::up(conn.db(), None)
        .await
        .expect("Failed to migrate database");

    let dev_mode = false;

    let client = reqwest_middleware::ClientBuilder::new(reqwest::Client::new()).build();

    let base_state = ServiceStateBase::new(
        conn.clone(),
        Arc::new(RedisConn::new(kv_conn)),
        nats_conn,
        dev_mode,
        (*&BASE_URL).clone(),
        client,
        None, // disabled fulltext search
    );
    let fed = FederationConfig::builder()
        .domain(MY_DOMAIN)
        .app_data(base_state.clone())
        .build()
        .await
        .expect("failed to build federation config");
    let app = ServiceState::new(base_state, fed);

    TestState { db, kv, app, nats }
}

pub const MY_DOMAIN: &str = "example.com";
pub const BASE_URL: Lazy<Url> = Lazy::new(|| Url::parse("https://example.com").unwrap());
