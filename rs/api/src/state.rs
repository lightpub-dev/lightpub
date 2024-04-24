use sqlx::SqlitePool;

use lightpub_backend::apub::queue::QueuedApubRequesterBuilder;
use lightpub_config::Config;

#[derive(Debug, Clone)]
pub struct AppState {
    pool: SqlitePool,
    queue: QueuedApubRequesterBuilder,
    config: Config,
    base_url: String,
}

impl AppState {
    pub fn new(pool: SqlitePool, queue: QueuedApubRequesterBuilder, config: Config) -> Self {
        let base_url = format!("{}://{}", config.http_scheme, config.hostname);
        Self {
            pool,
            queue,
            config,
            base_url,
        }
    }

    pub fn pool(&self) -> &SqlitePool {
        &self.pool
    }

    pub fn queue(&self) -> &QueuedApubRequesterBuilder {
        &self.queue
    }

    pub fn base_url(&self) -> &String {
        &self.base_url
    }

    pub fn config(&self) -> &Config {
        &self.config
    }
}
