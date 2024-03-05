use derive_more::Constructor;
use sqlx::MySqlPool;
use uuid::fmt::Simple;

use crate::services::{
    LocalUserFindError, LocalUserFinderService, PostCreateError, PostCreateService, ServiceError,
};

#[derive(Debug, Constructor)]
pub struct DBPostCreateService<T: LocalUserFinderService> {
    pool: MySqlPool,
    finder: T,
}

impl<T: LocalUserFinderService> PostCreateService for DBPostCreateService<T> {
    async fn create_post(
        &mut self,
        req: &crate::services::PostCreateRequest,
    ) -> Result<(), crate::services::ServiceError<crate::services::PostCreateError>> {
        let poster = self
            .finder
            .find_user_by_specifier(&req.poster)
            .await
            .map_err(|e| match e {
                ServiceError::SpecificError(LocalUserFindError::UserNotFound) => {
                    ServiceError::SpecificError(PostCreateError::PosterNotFound)
                }
                ServiceError::MiscError(e) => e.into(),
            })?;

        let poster_id = poster.id;
        let content = &req.content;
        let privacy = req.privacy.to_db();

        let result = sqlx::query!(
            "INSERT INTO posts (poster_id, content, privacy) VALUES(?, ?, ?)",
            poster_id,
            content,
            privacy,
        )
        .execute(&self.pool)
        .await?;

        unimplemented!()
    }
}
