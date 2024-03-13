pub mod follow;
pub mod post;
pub mod user;

use sqlx::MySqlPool;

use crate::{config::Config, holder, new_id_getter_service};

use self::{post::DBPostCreateService, user::DBSignerService};

use super::{
    apub::{new_apub_follow_service, new_apub_renderer_service, new_apub_reqwester_service},
    AllUserFinderService, Holder, LocalUserFinderService, PostCreateService, SignerService,
    UserAuthService, UserCreateService, UserFollowService,
};

pub fn new_user_service(pool: MySqlPool) -> holder!(UserCreateService) {
    Holder::new(user::DBUserCreateService::new(pool))
}

pub fn new_auth_service(pool: MySqlPool) -> holder!(UserAuthService) {
    Holder::new(user::DBAuthService::new(pool))
}

pub fn new_local_user_finder_service(pool: MySqlPool) -> holder!(LocalUserFinderService) {
    Holder::new(user::DBLocalUserFinderService::new(pool))
}

pub fn new_all_user_finder_service(
    pool: MySqlPool,
    config: Config,
) -> holder!(AllUserFinderService) {
    Holder::new(user::DBAllUserFinderService::new(
        pool.clone(),
        new_apub_reqwester_service(),
        new_local_user_finder_service(pool),
        new_id_getter_service(config.clone()),
    ))
}

pub fn new_follow_service(pool: MySqlPool, config: Config) -> holder!(UserFollowService) {
    Holder::new(follow::DBUserFollowService::new(
        pool.clone(),
        new_all_user_finder_service(pool.clone(), config.clone()),
        new_apub_follow_service(pool.clone(), new_id_getter_service(config.clone())),
        new_apub_reqwester_service(),
        new_db_user_signer_service(pool.clone(), config.clone()),
        new_id_getter_service(config),
    ))
}

pub fn new_post_create_service(pool: MySqlPool, config: Config) -> holder!(PostCreateService) {
    Holder::new(DBPostCreateService::new(
        pool.clone(),
        new_all_user_finder_service(pool.clone(), config.clone()),
        new_apub_renderer_service(config.clone()),
        new_apub_reqwester_service(),
        new_db_user_signer_service(pool.clone(), config.clone()),
        new_id_getter_service(config),
    ))
}

pub fn new_db_user_signer_service(pool: MySqlPool, config: Config) -> holder!(SignerService) {
    Box::new(DBSignerService::new(
        pool.clone(),
        new_local_user_finder_service(pool.clone()),
        new_id_getter_service(config),
    ))
}
