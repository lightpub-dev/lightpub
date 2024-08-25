use super::user::UserId;

// AuthToken value object
#[derive(Debug, PartialEq, Eq, Clone)]
pub struct AuthToken {
    token: String,
    user_id: UserId,
}

impl AuthToken {
    pub fn new(token: impl Into<String>, user_id: UserId) -> Self {
        Self {
            token: token.into(),
            user_id,
        }
    }

    pub fn token(&self) -> &str {
        &self.token
    }

    pub fn user_id(&self) -> UserId {
        self.user_id
    }
}
