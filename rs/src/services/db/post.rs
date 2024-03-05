use derive_more::Constructor;
use sqlx::MySqlPool;
use uuid::fmt::Simple;

use crate::{
    services::{
        LocalUserFindError, LocalUserFinderService, PostCreateError, PostCreateService,
        ServiceError,
    },
    utils::generate_uuid,
};

#[derive(Debug, Constructor)]
pub struct DBPostCreateService<T: LocalUserFinderService> {
    pool: MySqlPool,
    finder: T,
}

pub fn new_post_create_service(
    pool: MySqlPool,
    finder: impl LocalUserFinderService,
) -> impl PostCreateService {
    DBPostCreateService::new(pool, finder)
}

impl<T: LocalUserFinderService> PostCreateService for DBPostCreateService<T> {
    async fn create_post(
        &mut self,
        req: &crate::services::PostCreateRequest,
    ) -> Result<Simple, crate::services::ServiceError<crate::services::PostCreateError>> {
        let poster = self
            .finder
            .find_user_by_specifier(&req.poster)
            .await
            .map_err(|e| match e {
                ServiceError::SpecificError(
                    LocalUserFindError::UserNotFound | LocalUserFindError::NotLocalUser,
                ) => ServiceError::SpecificError(PostCreateError::PosterNotFound),
                ServiceError::MiscError(e) => e.into(),
            })?;

        let post_id = generate_uuid();
        let post_id_str = post_id.to_string();
        let poster_id = poster.id;
        let content = &req.content;
        let privacy = req.privacy.to_db();
        let created_at = chrono::Utc::now().naive_utc();

        tracing::debug!("coming to here: {} {} {}", poster_id, content, privacy);
        sqlx::query!(
            "INSERT INTO posts (id, poster_id, content, privacy, created_at) VALUES(?, ?, ?, ?, ?)",
            post_id_str,
            poster_id,
            content,
            privacy,
            created_at
        )
        .execute(&self.pool)
        .await?;

        Ok(post_id)
    }
}
