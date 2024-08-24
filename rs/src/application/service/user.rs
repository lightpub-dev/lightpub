use dto::{AuthTokenData, UserData, UserIdData};

use crate::{
    domain::{
        factory::auth::AuthTokenFactory,
        model::{
            user::{Nickname, User, UserId, Username},
            DateTime,
        },
    },
    holder,
    repository::interface::uow::UnitOfWork,
};

use super::id::IDGenerationService;

pub struct UserApplicationService {
    uow: holder!(UnitOfWork),
}

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

        self.uow
            .repository_manager()
            .user_repository()
            .create(&new_user)
            .await?;

        self.uow.commit().await?;

        Ok(UserIdData::from_user_id(new_user.id()))
    }

    pub async fn get_user_by_id(
        &mut self,
        user_id: &str,
    ) -> Result<Option<UserData>, anyhow::Error> {
        todo!()
    }

    pub async fn get_user_id_by_username_and_host(
        &mut self,
        username: &str,
        host: Option<&str>,
    ) -> Result<Option<UserIdData>, anyhow::Error> {
        let mut user_repository = self.uow.repository_manager().user_repository();
        let username = Username::from_str(username).unwrap();
        let id = user_repository
            .find_by_username_and_host(&username, host)
            .await
            .map(|user| user.map(|user| UserIdData::from_user_id(user.id())))?;

        self.uow.commit().await?;

        Ok(id)
    }

    pub async fn id_exists(&mut self, user_id: &str) -> Result<bool, anyhow::Error> {
        let user = self.get_user_by_id(user_id).await?;
        Ok(user.is_some())
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
            .uow
            .repository_manager()
            .user_repository()
            .find_by_username_and_host(&username, None)
            .await?;

        match user {
            None => {
                // throw error
                self.uow.rollback().await?;
                panic!("user not found");
            }
            Some(user) => {
                // user found

                if !user.validate_password(plain_passwd) {
                    // throw error
                    self.uow.rollback().await?;
                    panic!("invalid password");
                }

                // password is correct
                let token = self.auth_token_factory.create();
                self.uow
                    .repository_manager()
                    .auth_token_repository()
                    .create(&token)
                    .await?;
                self.uow.commit().await?;
                Ok(AuthTokenData::new(token.token().to_string()))
            }
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
}
