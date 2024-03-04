use sqlx::MySqlPool;

use crate::models::User;

pub mod user;

pub fn new_user_service(pool: MySqlPool) -> user::DBUserService {
    user::DBUserService::new(pool)
}
