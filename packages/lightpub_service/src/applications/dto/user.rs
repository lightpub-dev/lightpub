use crate::domain::models::user::{Nickname, UserPassword, Username};

#[derive(Debug, Clone)]
pub struct UserRegisterDto {
    pub username: Username,
    pub password: UserPassword,
    pub nickname: Nickname,
}
