use sqlx::MySqlPool;

#[derive(Debug)]
pub struct DBQueueService {
    #[allow(dead_code)]
    pool: MySqlPool,
}
