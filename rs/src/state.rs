use sqlx::MySqlPool;

#[derive(Debug, Clone)]
pub struct AppState {
    pool: MySqlPool,
    base_url: String,
}

impl AppState {
    pub fn new(pool: MySqlPool, base_url: String) -> Self {
        Self { pool, base_url }
    }

    pub fn pool(&self) -> &MySqlPool {
        &self.pool
    }

    pub fn base_url(&self) -> &String {
        &self.base_url
    }
}
