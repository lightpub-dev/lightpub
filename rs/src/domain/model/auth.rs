// AuthToken value object
#[derive(Debug, PartialEq, Eq, Clone)]
pub struct AuthToken {
    token: String,
}

impl AuthToken {
    pub fn from_str(token: String) -> Self {
        Self { token }
    }

    pub fn token(&self) -> &str {
        &self.token
    }
}
