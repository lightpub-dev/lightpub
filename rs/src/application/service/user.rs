use dto::AuthTokenData;

use crate::{
    domain::{
        factory::auth::AuthTokenFactory,
        model::{
            user::{Nickname, User, UserId, Username},
            DateTime,
        },
    },
    holder,
    repository::interface::{auth::AuthTokenRepository, user::UserRepository},
};

use super::id::IDGenerationService;

pub struct UserApplicationService {
    user_repository: holder!(UserRepository),
}

pub struct UserSecurityApplicationService {
    user_repository: holder!(UserRepository),
    auth_token_repository: holder!(AuthTokenRepository),
    auth_token_factory: holder!(AuthTokenFactory),
}

impl UserApplicationService {
    pub async fn create_user(
        &mut self,
        username: &str,
        nickname: &str,
        passwd: &str,
    ) -> Result<(), anyhow::Error> {
        let mut new_user = User::new(
            UserId::from_uuid(IDGenerationService::generate_id()),
            Username::from_str(username).unwrap(),
            Nickname::from_str(nickname).unwrap(),
            DateTime::now(),
        );

        if !new_user.set_password(passwd) {
            // throw error
            panic!("failed to set passwd");
        }

        self.user_repository.create(&new_user).await?;

        Ok(())
    }
}

impl UserSecurityApplicationService {
    pub async fn login(
        &mut self,
        username: &str,
        plain_passwd: &str,
    ) -> Result<AuthTokenData, anyhow::Error> {
        let username = Username::from_str(username).unwrap();
        let user = self
            .user_repository
            .find_by_username_and_host(&username, None)
            .await?;

        match user {
            None => {
                // throw error
                panic!("user not found");
            }
            Some(user) => {
                if !user.validate_password(plain_passwd) {
                    // throw error
                    panic!("invalid password");
                }

                let token = self.auth_token_factory.create();
                self.auth_token_repository.create(&token).await?;
                Ok(AuthTokenData::new(token.token().to_string()))
            }
        }
    }
}

mod dto {
    pub struct AuthTokenData {
        token: String,
    }

    impl AuthTokenData {
        pub fn new(token: String) -> Self {
            Self { token }
        }

        pub fn token(&self) -> &str {
            &self.token
        }
    }
}
