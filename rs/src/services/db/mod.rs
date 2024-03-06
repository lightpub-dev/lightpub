pub mod follow;
pub mod post;
pub mod user;

use sqlx::MySqlPool;

use super::{LocalUserFinderService, UserAuthService, UserCreateService, UserFollowService};

pub fn new_user_service(pool: MySqlPool) -> impl UserCreateService {
    user::DBUserCreateService::new(pool)
}

pub fn new_auth_service(pool: MySqlPool) -> impl UserAuthService {
    user::DBAuthService::new(pool)
}

pub fn new_local_user_finder_service(pool: MySqlPool) -> impl LocalUserFinderService {
    user::DBLocalUserFinderService::new(pool)
}

pub fn new_follow_service(pool: MySqlPool) -> impl UserFollowService {
    follow::DBUserFollowService::new(pool.clone(), new_local_user_finder_service(pool))
}
