use sqlx::MySqlPool;

#[derive(Debug)]
pub struct DBQueueService {
    pool: MySqlPool,
}
