pub mod post;
pub mod user;

use sqlx::MySqlPool;

use super::{LocalUserFinderService, UserAuthService, UserCreateService};

pub fn new_user_service(pool: MySqlPool) -> impl UserCreateService {
    user::DBUserCreateService::new(pool)
}

pub fn new_auth_service(pool: MySqlPool) -> impl UserAuthService {
    user::DBAuthService::new(pool)
}

pub fn new_local_user_finder_service(pool: MySqlPool) -> impl LocalUserFinderService {
    user::DBLocalUserFinderService::new(pool)
}
