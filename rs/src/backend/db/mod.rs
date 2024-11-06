pub mod follow;
pub mod key;
pub mod post;
pub mod trend;
pub mod upload;
pub mod user;

use sqlx::SqlitePool;

use self::{key::DBKeyFetcher, post::DBPostCreateService, user::DBSignerService};
use crate::backend::id::IDGetterService;
use crate::config::Config;
use crate::holder;
use crate::utils::key::KeyFetcher;

use super::{
    apub::{
        new_apub_follow_service, new_apub_renderer_service, new_apub_reqester_service,
        post::PostContentService,
    },
    AllUserFinderService, Holder, LocalUserFinderService, PostCreateService, SignerService,
    TrendService, UploadService, UserAuthService, UserCreateService, UserFollowService,
    UserPostService, UserProfileService,
};

pub fn new_id_getter_service(config: Config) -> IDGetterService {
    IDGetterService::new(config)
}

pub fn new_user_service(pool: SqlitePool) -> holder!(UserCreateService) {
    Holder::new(user::DBUserCreateService::new(pool))
}

pub fn new_auth_service(pool: SqlitePool) -> holder!(UserAuthService) {
    Holder::new(user::DBAuthService::new(pool))
}

pub fn new_local_user_finder_service(pool: SqlitePool) -> holder!(LocalUserFinderService) {
    Holder::new(user::DBLocalUserFinderService::new(pool))
}

pub fn new_all_user_finder_service(
    pool: SqlitePool,
    config: Config,
) -> holder!(AllUserFinderService) {
    Holder::new(user::DBAllUserFinderService::new(
        pool.clone(),
        new_apub_reqester_service(pool.clone(), &config),
        new_local_user_finder_service(pool),
        new_id_getter_service(config.clone()),
    ))
}

pub fn new_follow_service(pool: SqlitePool, config: Config) -> holder!(UserFollowService) {
    Holder::new(follow::DBUserFollowService::new(
        pool.clone(),
        new_all_user_finder_service(pool.clone(), config.clone()),
        new_apub_follow_service(
            pool.clone(),
            new_id_getter_service(config.clone()),
            new_all_user_finder_service(pool.clone(), config.clone()),
        ),
        new_apub_reqester_service(pool.clone(), &config),
        new_db_user_signer_service(pool.clone(), config.clone()),
        new_id_getter_service(config),
    ))
}

pub fn new_post_create_service(pool: SqlitePool, config: Config) -> holder!(PostCreateService) {
    Holder::new(DBPostCreateService::new(
        pool.clone(),
        new_all_user_finder_service(pool.clone(), config.clone()),
        new_apub_renderer_service(config.clone()),
        new_apub_reqester_service(pool.clone(), &config),
        new_db_user_signer_service(pool.clone(), config.clone()),
        new_id_getter_service(config.clone()),
        new_post_content_service(),
        new_follow_service(pool, config),
    ))
}

pub fn new_db_user_signer_service(pool: SqlitePool, config: Config) -> holder!(SignerService) {
    Box::new(DBSignerService::new(
        pool.clone(),
        new_local_user_finder_service(pool.clone()),
        new_id_getter_service(config),
    ))
}

pub fn new_post_content_service() -> PostContentService {
    PostContentService::new()
}

pub fn new_db_key_fetcher_service(pool: SqlitePool, config: Config) -> holder!(KeyFetcher) {
    Box::new(DBKeyFetcher::new(
        pool.clone(),
        new_all_user_finder_service(pool, config),
    ))
}

pub fn new_db_file_upload_service(pool: SqlitePool, _config: Config) -> holder!(UploadService) {
    Box::new(upload::DBUploadService::new(
        pool.clone(),
        new_local_user_finder_service(pool),
    ))
}

pub fn new_db_user_profile_service(
    pool: SqlitePool,
    _config: Config,
) -> holder!(UserProfileService) {
    Box::new(user::DBUserProfileService::new(
        pool.clone(),
        new_local_user_finder_service(pool.clone()),
    ))
}

pub fn new_db_user_post_service(pool: SqlitePool, config: Config) -> holder!(UserPostService) {
    Box::new(post::DBUserPostService::new(
        pool.clone(),
        new_all_user_finder_service(pool.clone(), config.clone()),
        new_id_getter_service(config),
    ))
}

pub fn new_db_trend_service(pool: SqlitePool) -> holder!(TrendService) {
    Box::new(trend::DBTrendService::new(pool.clone()))
}
