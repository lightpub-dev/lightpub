use sea_orm_migration::{prelude::*, schema::*};

use crate::m20250211_132721_uploads::Upload;

#[derive(DeriveMigrationName)]
pub struct Migration;

#[async_trait::async_trait]
impl MigrationTrait for Migration {
    async fn up(&self, manager: &SchemaManager) -> Result<(), DbErr> {
        // Replace the sample below with your own migration scripts
        manager
            .alter_table(
                TableAlterStatement::new()
                    .table(Upload::Table)
                    .add_column(string_len(Upload::MimeType, 255))
                    .to_owned(),
            )
            .await?;

        Ok(())
    }

    async fn down(&self, manager: &SchemaManager) -> Result<(), DbErr> {
        // Replace the sample below with your own migration scripts
        manager
            .alter_table(
                TableAlterStatement::new()
                    .table(Upload::Table)
                    .drop_column(Upload::MimeType)
                    .to_owned(),
            )
            .await?;

        Ok(())
    }
}
