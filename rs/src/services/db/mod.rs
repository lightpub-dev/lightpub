pub mod post;
pub mod user;

use sqlx::MySqlPool;

use super::UserCreateService;

pub fn new_user_service(pool: MySqlPool) -> impl UserCreateService {
    user::DBUserService::new(pool)
}
