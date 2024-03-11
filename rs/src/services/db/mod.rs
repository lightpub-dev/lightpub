pub mod follow;
pub mod post;
pub mod user;

use sqlx::MySqlPool;

use crate::{config::Config, new_id_getter_service};

use self::{post::DBPostCreateService, user::new_db_user_signer_service};

use super::{
    apub::{new_apub_follow_service, new_apub_renderer_service, new_apub_reqwester_service},
    AllUserFinderService, ApubRequestService, LocalUserFinderService, PostCreateService,
    UserAuthService, UserCreateService, UserFollowService,
};

pub fn new_user_service(pool: MySqlPool) -> impl UserCreateService {
    user::DBUserCreateService::new(pool)
}

pub fn new_auth_service(pool: MySqlPool) -> impl UserAuthService {
    user::DBAuthService::new(pool)
}

pub fn new_local_user_finder_service(pool: MySqlPool) -> impl LocalUserFinderService {
    user::DBLocalUserFinderService::new(pool)
}

pub fn new_all_user_finder_service(
    pool: MySqlPool,
    req: impl ApubRequestService,
    finder: impl LocalUserFinderService,
    config: Config,
) -> impl AllUserFinderService {
    user::DBAllUserFinderService::new(pool, req, finder, new_id_getter_service(config.clone()))
}

pub fn new_follow_service(pool: MySqlPool, config: Config) -> impl UserFollowService {
    follow::DBUserFollowService::new(
        pool.clone(),
        new_all_user_finder_service(
            pool.clone(),
            new_apub_reqwester_service(),
            new_local_user_finder_service(pool.clone()),
            config.clone(),
        ),
        new_apub_follow_service(pool.clone(), new_id_getter_service(config.clone())),
        new_apub_reqwester_service(),
        new_db_user_signer_service(
            pool.clone(),
            new_local_user_finder_service(pool),
            new_id_getter_service(config.clone()),
        ),
        new_id_getter_service(config),
    )
}

pub fn new_post_create_service(pool: MySqlPool, config: Config) -> impl PostCreateService {
    DBPostCreateService::new(
        pool.clone(),
        new_all_user_finder_service(
            pool.clone(),
            new_apub_reqwester_service(),
            new_local_user_finder_service(pool.clone()),
            config.clone(),
        ),
        new_apub_renderer_service(config.clone()),
        new_apub_reqwester_service(),
        new_db_user_signer_service(
            pool.clone(),
            new_local_user_finder_service(pool.clone()),
            new_id_getter_service(config),
        ),
    )
}
