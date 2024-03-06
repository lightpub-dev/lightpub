use sqlx::MySqlPool;

use crate::config::Config;

#[derive(Debug, Clone)]
pub struct AppState {
    pool: MySqlPool,
    config: Config,
    base_url: String,
}

impl AppState {
    pub fn new(pool: MySqlPool, config: Config) -> Self {
        let base_url = format!("{}://{}", config.http_scheme, config.hostname);
        Self {
            pool,
            config,
            base_url,
        }
    }

    pub fn pool(&self) -> &MySqlPool {
        &self.pool
    }

    pub fn base_url(&self) -> &String {
        &self.base_url
    }

    pub fn config(&self) -> &Config {
        &self.config
    }
}
