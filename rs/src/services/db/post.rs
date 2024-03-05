use sqlx::MySqlPool;
use uuid::fmt::Simple;

#[derive(Debug)]
pub struct DBUserService {
    pool: MySqlPool,
}

impl DBUserService {
    pub fn new(pool: MySqlPool) -> Self {
        Self { pool }
    }
}

#[derive(Debug, Clone, PartialEq, Eq)]
pub enum PostPrivacy {
    Public,
    Unlisted,
    Followers,
    Private,
}

#[derive(Debug, Clone)]
pub struct PostCreateRequest {
    poster_id: Simple,
    content: String,
    privacy: PostPrivacy,
}
