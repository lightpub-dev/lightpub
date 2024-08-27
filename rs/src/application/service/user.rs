use derive_builder::Builder;
use derive_more::Constructor;
use dto::{AuthTokenData, UserData, UserIdData};
use thiserror::Error;

use crate::{
    domain::{
        factory::auth::AuthTokenFactory,
        model::{
            user::{Nickname, User, UserId, Username},
            DateTime,
        },
        service::user::UserService,
    },
    holder,
    repository::interface::uow::UnitOfWork,
};

use super::id::IDGenerationService;

#[derive(Constructor)]
pub struct UserApplicationService {
    uow: holder!(UnitOfWork),
}

#[derive(Constructor)]
pub struct UserSecurityApplicationService {
    uow: holder!(UnitOfWork),
    auth_token_factory: holder!(AuthTokenFactory),
}

impl UserApplicationService {
    pub async fn create_user(
        &mut self,
        username: &str,
        nickname: &str,
        passwd: &str,
    ) -> Result<UserIdData, anyhow::Error> {
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

        {
            let mut repo = self.uow.repository_manager().await?;
            repo.user_repository().create(&new_user).await?;
        }

        self.uow.commit().await?;

        Ok(UserIdData::from_user_id(new_user.id()))
    }

    pub async fn get_user_by_id(
        &mut self,
        user_id: &str,
        get_user_options: &GetUserOptions,
    ) -> Result<Option<UserData>, anyhow::Error> {
        todo!()
    }

    pub async fn get_user_id_by_username_and_host(
        &mut self,
        username: &str,
        host: Option<&str>,
    ) -> Result<Option<UserIdData>, anyhow::Error> {
        let id = {
            let mut repo = self.uow.repository_manager().await?;
            let username = Username::from_str(username).unwrap();
            let id = repo
                .user_repository()
                .find_by_username_and_host(&username, host)
                .await
                .map(|user| user.map(|user| UserIdData::from_user_id(user.id())))?;
            id
        };

        self.uow.commit().await?;

        Ok(id)
    }

    pub async fn id_exists(&mut self, user_id: &str) -> Result<bool, anyhow::Error> {
        let user = self
            .get_user_by_id(user_id, &GetUserOptions::default())
            .await?;
        Ok(user.is_some())
    }
}

#[derive(Builder)]
pub struct GetUserOptions {
    fill_uris: bool,
}

impl Default for GetUserOptions {
    fn default() -> Self {
        Self { fill_uris: false }
    }
}

#[derive(Error, Debug)]
pub enum LoginError {
    #[error("User not found")]
    UserNotFound,
    #[error("Invalid password")]
    InvalidPassword,
}

impl UserSecurityApplicationService {
    pub async fn login(
        &mut self,
        username: &str,
        plain_passwd: &str,
    ) -> Result<AuthTokenData, anyhow::Error> {
        let username = Username::from_str(username).unwrap();

        let token = {
            let mut repo = self.uow.repository_manager().await?;
            let user = repo
                .user_repository()
                .find_by_username_and_host(&username, None)
                .await?;

            match user {
                None => {
                    // throw error
                    Err(LoginError::UserNotFound)
                }
                Some(user) => {
                    // user found

                    if !user.validate_password(plain_passwd) {
                        // throw error
                        Err(LoginError::InvalidPassword)
                    } else {
                        // password is correct
                        let token = self.auth_token_factory.create(user.id());
                        repo.auth_token_repository()
                            .create(&token, user.id())
                            .await?;
                        Ok(token.token().to_string())
                    }
                }
            }
        };
        self.uow.commit().await?;

        match token {
            Err(e) => Err(e.into()),
            Ok(token) => Ok(AuthTokenData::new(token)),
        }
    }

    pub async fn validate_token(
        &mut self,
        token: &str,
    ) -> Result<Option<UserIdData>, anyhow::Error> {
        let token = {
            let mut repo = self.uow.repository_manager().await?;
            let token = repo.auth_token_repository().find_by_token(token).await?;
            token
        };

        self.uow.commit().await?;

        match token {
            None => Ok(None),
            Some(token) => Ok(Some(UserIdData::from_user_id(token.user_id()))),
        }
    }
}

mod dto {
    use crate::domain::model::user::UserId;

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

    #[derive(Debug, Clone, Copy, PartialEq, Eq)]
    pub struct UserIdData(UserId);

    impl UserIdData {
        pub fn from_user_id(user_id: UserId) -> Self {
            Self(user_id)
        }

        pub fn user_id(&self) -> UserId {
            self.0
        }
    }

    #[derive(Debug, Clone)]
    pub struct UserData {}

    impl UserData {
        pub fn set_inbox(&mut self, inbox: impl Into<String>) {
            todo!()
        }

        pub fn set_outbox(&mut self, outbox: impl Into<String>) {
            todo!()
        }

        pub fn set_shared_inbox(&mut self, shared_inbox: impl Into<String>) {
            todo!()
        }

        pub fn set_uri(&mut self, uri: impl Into<String>) {
            todo!()
        }

        pub fn inbox(&self) -> Option<&str> {
            todo!()
        }

        pub fn outbox(&self) -> Option<&str> {
            todo!()
        }

        pub fn shared_inbox(&self) -> Option<&str> {
            todo!()
        }

        pub fn uri(&self) -> Option<&str> {
            todo!()
        }
    }
}
