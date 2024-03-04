use sqlx::MySqlPool;

#[derive(Debug, Clone)]
pub struct AppState {
    pool: MySqlPool,
}

impl AppState {
    pub fn new(pool: MySqlPool) -> Self {
        Self { pool }
    }

    pub fn pool(&self) -> &MySqlPool {
        &self.pool
    }
}
