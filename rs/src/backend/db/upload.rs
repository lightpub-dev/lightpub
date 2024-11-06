use async_trait::async_trait;
use derive_more::Constructor;
use gen_span::gen_span;
use sqlx::SqlitePool;
use uuid::fmt::Simple;

use crate::backend::{LocalUserFinderService, UploadService};
use crate::holder;
use crate::model::UserSpecifier;

#[derive(Constructor)]
pub struct DBUploadService {
    pool: SqlitePool,
    finder: holder!(LocalUserFinderService),
}

#[gen_span]
#[async_trait]
impl UploadService for DBUploadService {
    async fn upload_file(
        &mut self,
        user: &UserSpecifier,
        file_id: Simple,
        file_ext: &str,
    ) -> Result<(), anyhow::Error> {
        let user_id = self.finder.find_user_by_specifier(user).await?.id;

        let file_id_str = file_id.to_string();
        let user_id_str = user_id.to_string();
        sqlx::query!(
            r#"
            INSERT INTO uploaded_files (id, file_ext, uploaded_by_id) VALUES (?, ?, ?)
        "#,
            file_id_str,
            file_ext,
            user_id_str
        )
        .execute(&self.pool)
        .await?;

        Ok(())
    }
}
