use sea_orm_migration::{prelude::*, schema::timestamp_with_time_zone_null};

use crate::m20250202_050205_notes::Note;

#[derive(DeriveMigrationName)]
pub struct Migration;

#[async_trait::async_trait]
impl MigrationTrait for Migration {
    async fn up(&self, manager: &SchemaManager) -> Result<(), DbErr> {
        // Replace the sample below with your own migration scripts
        manager
            .alter_table(
                TableAlterStatement::new()
                    .table(Note::Table)
                    .add_column(timestamp_with_time_zone_null(Note::FetchedAt))
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
                    .table(Note::Table)
                    .drop_column(Note::FetchedAt)
                    .to_owned(),
            )
            .await?;

        Ok(())
    }
}
