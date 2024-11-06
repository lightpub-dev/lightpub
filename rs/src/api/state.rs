use sqlx::SqlitePool;

use crate::config::Config;

#[derive(Debug, Clone)]
pub struct AppState {
    pool: SqlitePool,
    config: Config,
    base_url: String,
}

impl AppState {
    pub fn new(pool: SqlitePool, config: Config) -> Self {
        let base_url = format!(
            "{}://{}{}",
            config.http_scheme, config.hostname, config.base_path
        );
        Self {
            pool,
            config,
            base_url,
        }
    }

    pub fn pool(&self) -> &SqlitePool {
        &self.pool
    }

    pub fn base_url(&self) -> &String {
        &self.base_url
    }

    pub fn config(&self) -> &Config {
        &self.config
    }
}
