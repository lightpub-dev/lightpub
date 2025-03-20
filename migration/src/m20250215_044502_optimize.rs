use sea_orm_migration::prelude::*;

use crate::m20250202_050205_notes::Note;

#[derive(DeriveMigrationName)]
pub struct Migration;

#[async_trait::async_trait]
impl MigrationTrait for Migration {
    async fn up(&self, manager: &SchemaManager) -> Result<(), DbErr> {
        // Replace the sample below with your own migration scripts
        manager
            .create_index(
                IndexCreateStatement::new()
                    .name("idx_note_created_at")
                    .table(Note::Table)
                    .col((Note::CreatedAt, IndexOrder::Desc))
                    .to_owned(),
            )
            .await?;

        manager
            .create_index(
                IndexCreateStatement::new()
                    .name("idx_note_reply_to_id")
                    .table(Note::Table)
                    .col(Note::ReplyToId)
                    .to_owned(),
            )
            .await?;

        manager
            .create_index(
                IndexCreateStatement::new()
                    .name("idx_note_renote_of_id")
                    .table(Note::Table)
                    .col(Note::RenoteOfId)
                    .to_owned(),
            )
            .await?;

        Ok(())
    }

    async fn down(&self, manager: &SchemaManager) -> Result<(), DbErr> {
        // Replace the sample below with your own migration scripts
        manager
            .drop_index(
                IndexDropStatement::new()
                    .name("idx_note_created_at")
                    .table(Note::Table)
                    .to_owned(),
            )
            .await?;

        manager
            .drop_index(
                IndexDropStatement::new()
                    .name("idx_note_reply_to_id")
                    .table(Note::Table)
                    .to_owned(),
            )
            .await?;

        manager
            .drop_index(
                IndexDropStatement::new()
                    .name("idx_note_renote_of_id")
                    .table(Note::Table)
                    .to_owned(),
            )
            .await?;

        Ok(())
    }
}
