use rand::{distributions::Alphanumeric, rngs::OsRng, Rng};

use crate::domain::model::auth::AuthToken;

pub trait AuthTokenFactory {
    fn create(&mut self) -> AuthToken;
}

pub struct DefaultAuthTokenFactory {}

impl AuthTokenFactory for DefaultAuthTokenFactory {
    fn create(&mut self) -> AuthToken {
        let chars = OsRng
            .sample_iter(&Alphanumeric)
            .take(64)
            .map(char::from)
            .collect();
        AuthToken::from_str(chars)
    }
}
