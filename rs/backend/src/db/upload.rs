use async_trait::async_trait;
use derive_more::Constructor;
use sqlx::MySqlPool;
use uuid::fmt::Simple;

use crate::{holder, LocalUserFinderService, UploadService};
use lightpub_model::UserSpecifier;

#[derive(Constructor)]
pub struct DBUploadService {
    pool: MySqlPool,
    finder: holder!(LocalUserFinderService),
}

#[async_trait]
impl UploadService for DBUploadService {
    async fn upload_file(
        &mut self,
        user: &UserSpecifier,
        file_id: Simple,
        file_ext: &str,
    ) -> Result<(), anyhow::Error> {
        let user_id = self.finder.find_user_by_specifier(user).await?.id;

        sqlx::query!(
            r#"
            INSERT INTO uploaded_files (id, file_ext, uploaded_by_id) VALUES (?, ?, ?)
        "#,
            file_id.to_string(),
            file_ext,
            user_id.to_string()
        )
        .execute(&self.pool)
        .await?;

        Ok(())
    }
}
