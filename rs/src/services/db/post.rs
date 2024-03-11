use derive_more::Constructor;
use sqlx::MySqlPool;
use uuid::fmt::Simple;

use crate::{
    services::{
        LocalUserFindError, LocalUserFinderService, PostCreateError, PostCreateRequest,
        PostCreateService, ServiceError,
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
        use PostCreateRequest::*;
        let (repost_of_id, reply_to_id, content) = match req {
            Normal(r) => (None, None, r.content.clone().into()),
            Repost(r) => (r.repost_of.into(), None, None),
            Quote(r) => (r.repost_of.into(), None, r.content.clone().into()),
            Reply(r) => (None, r.reply_to.into(), r.content.clone().into()),
        };
        let poster = req.poster();

        let poster = self
            .finder
            .find_user_by_specifier(&poster)
            .await
            .map_err(|e| match e {
                ServiceError::SpecificError(
                    LocalUserFindError::UserNotFound | LocalUserFindError::NotLocalUser,
                ) => ServiceError::SpecificError(PostCreateError::PosterNotFound),
                ServiceError::MiscError(e) => e.into(),
            })?;

        if let Some(repost_of_id) = repost_of_id {
            // check if the post exists
            let repost_target =
                sqlx::query!("SELECT id FROM posts WHERE id=?", repost_of_id.to_string())
                    .fetch_optional(&self.pool)
                    .await?;
            if repost_target.is_none() {
                return Err(ServiceError::from_se(PostCreateError::RepostOfNotFound));
            }
        }

        if let Some(reply_to_id) = reply_to_id {
            // check if the post exists
            let reply_target =
                sqlx::query!("SELECT id FROM posts WHERE id=?", reply_to_id.to_string())
                    .fetch_optional(&self.pool)
                    .await?;
            if reply_target.is_none() {
                return Err(ServiceError::from_se(PostCreateError::ReplyToNotFound));
            }
        }

        let post_id = generate_uuid();
        let post_id_str = post_id.to_string();
        let poster_id = poster.id;
        let privacy = req.privacy().to_db();
        let created_at = chrono::Utc::now().naive_utc();

        sqlx::query!(
            "INSERT INTO posts (id, poster_id, content, privacy, created_at, repost_of_id, reply_to_id) VALUES(?, ?, ?, ?, ?, ?, ?)",
            post_id_str,
            poster_id,
            content,
            privacy,
            created_at,
            repost_of_id.map(|s|s.to_string()),
            reply_to_id.map(|s|s.to_string())
        )
        .execute(&self.pool)
        .await?;

        Ok(post_id)
    }
}
