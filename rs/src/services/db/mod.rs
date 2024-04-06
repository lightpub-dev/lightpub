pub mod follow;
pub mod key;
pub mod post;
pub mod upload;
pub mod user;

use sqlx::MySqlPool;

use crate::{config::Config, holder, new_id_getter_service, utils::key::KeyFetcher};

use self::{key::DBKeyFetcher, post::DBPostCreateService, user::DBSignerService};

use super::{
    apub::{
        new_apub_follow_service, new_apub_renderer_service, new_apub_reqwester_service,
        post::PostContentService, queue::QueuedApubRequesterBuilder,
    },
    AllUserFinderService, Holder, LocalUserFinderService, PostCreateService, SignerService,
    UploadService, UserAuthService, UserCreateService, UserFollowService, UserPostService,
    UserProfileService,
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
    queue: QueuedApubRequesterBuilder,
    config: Config,
) -> holder!(AllUserFinderService) {
    Holder::new(user::DBAllUserFinderService::new(
        pool.clone(),
        new_apub_reqwester_service(queue, &config),
        new_local_user_finder_service(pool),
        new_id_getter_service(config.clone()),
    ))
}

pub fn new_follow_service(
    pool: MySqlPool,
    queue: QueuedApubRequesterBuilder,
    config: Config,
) -> holder!(UserFollowService) {
    Holder::new(follow::DBUserFollowService::new(
        pool.clone(),
        new_all_user_finder_service(pool.clone(), queue.clone(), config.clone()),
        new_apub_follow_service(
            pool.clone(),
            new_id_getter_service(config.clone()),
            new_local_user_finder_service(pool.clone()),
        ),
        new_apub_reqwester_service(queue, &config),
        new_db_user_signer_service(pool.clone(), config.clone()),
        new_id_getter_service(config),
    ))
}

pub fn new_post_create_service(
    pool: MySqlPool,
    queue: QueuedApubRequesterBuilder,
    config: Config,
) -> holder!(PostCreateService) {
    Holder::new(DBPostCreateService::new(
        pool.clone(),
        new_all_user_finder_service(pool.clone(), queue.clone(), config.clone()),
        new_apub_renderer_service(config.clone()),
        new_apub_reqwester_service(queue.clone(), &config),
        new_db_user_signer_service(pool.clone(), config.clone()),
        new_id_getter_service(config.clone()),
        new_post_content_service(),
        new_follow_service(pool, queue, config),
    ))
}

pub fn new_db_user_signer_service(pool: MySqlPool, config: Config) -> holder!(SignerService) {
    Box::new(DBSignerService::new(
        pool.clone(),
        new_local_user_finder_service(pool.clone()),
        new_id_getter_service(config),
    ))
}

pub fn new_post_content_service() -> PostContentService {
    PostContentService::new()
}

pub fn new_db_key_fetcher_service(
    pool: MySqlPool,
    queue: QueuedApubRequesterBuilder,
    config: Config,
) -> holder!(KeyFetcher) {
    Box::new(DBKeyFetcher::new(
        pool.clone(),
        new_all_user_finder_service(pool, queue, config),
    ))
}

pub fn new_db_file_upload_service(pool: MySqlPool, _config: Config) -> holder!(UploadService) {
    Box::new(upload::DBUploadService::new(
        pool.clone(),
        new_local_user_finder_service(pool),
    ))
}

pub fn new_db_user_profile_service(
    pool: MySqlPool,
    _config: Config,
) -> holder!(UserProfileService) {
    Box::new(user::DBUserProfileService::new(
        pool.clone(),
        new_local_user_finder_service(pool.clone()),
    ))
}

pub fn new_db_user_post_service(
    pool: MySqlPool,
    queue: QueuedApubRequesterBuilder,
    config: Config,
) -> holder!(UserPostService) {
    Box::new(post::DBUserPostService::new(
        pool.clone(),
        new_all_user_finder_service(pool.clone(), queue, config.clone()),
        new_id_getter_service(config),
    ))
}
