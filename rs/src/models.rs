pub struct User {
    pub id: sqlx::types::uuid::fmt::Simple,
    pub username: String,
    pub host: Option<String>,
    pub bpasswd: Option<String>,
    pub nickname: String,
    pub bio: String,
    pub uri: Option<String>,
    pub shared_inbox: Option<String>,
    pub inbox: Option<String>,
    pub outbox: Option<String>,
    pub private_key: Option<String>,
    pub public_key: Option<String>,
    pub created_at: chrono::DateTime<chrono::Utc>,
}