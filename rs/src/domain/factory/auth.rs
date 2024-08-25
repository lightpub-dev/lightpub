use rand::{distributions::Alphanumeric, rngs::OsRng, Rng};

use crate::domain::model::{auth::AuthToken, user::UserId};

pub trait AuthTokenFactory {
    fn create(&mut self, user_id: UserId) -> AuthToken;
}

pub struct DefaultAuthTokenFactory {}

impl AuthTokenFactory for DefaultAuthTokenFactory {
    fn create(&mut self, user_id: UserId) -> AuthToken {
        let chars: String = OsRng
            .sample_iter(&Alphanumeric)
            .take(64)
            .map(char::from)
            .collect();
        AuthToken::new(chars, user_id)
    }
}
