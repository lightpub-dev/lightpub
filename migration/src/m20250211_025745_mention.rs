use sea_orm_migration::{prelude::*, schema::*};

use crate::{m20220101_000001_create_table::User, m20250202_050205_notes::Note};

#[derive(DeriveMigrationName)]
pub struct Migration;

#[async_trait::async_trait]
impl MigrationTrait for Migration {
    async fn up(&self, manager: &SchemaManager) -> Result<(), DbErr> {
        // Replace the sample below with your own migration scripts
        manager
            .create_table(
                TableCreateStatement::new()
                    .table(NoteMention::Table)
                    .col(pk_auto(NoteMention::Id))
                    .col(uuid(NoteMention::NoteId))
                    .col(uuid(NoteMention::TargetUserId))
                    .to_owned(),
            )
            .await?;

        manager
            .create_foreign_key(
                ForeignKeyCreateStatement::new()
                    .name("fk_note_mention_note_id")
                    .from(NoteMention::Table, NoteMention::NoteId)
                    .to(Note::Table, Note::Id)
                    .on_delete(ForeignKeyAction::Cascade)
                    .on_update(ForeignKeyAction::Cascade)
                    .to_owned(),
            )
            .await?;

        manager
            .create_foreign_key(
                ForeignKeyCreateStatement::new()
                    .name("fk_note_mention_user_id")
                    .from(NoteMention::Table, NoteMention::TargetUserId)
                    .to(User::Table, User::Id)
                    .on_delete(ForeignKeyAction::Cascade)
                    .on_update(ForeignKeyAction::Cascade)
                    .to_owned(),
            )
            .await?;

        manager
            .create_index(
                IndexCreateStatement::new()
                    .name("idx_note_mention_unique")
                    .table(NoteMention::Table)
                    .col(NoteMention::NoteId)
                    .col(NoteMention::TargetUserId)
                    .unique()
                    .to_owned(),
            )
            .await?;

        Ok(())
    }

    async fn down(&self, manager: &SchemaManager) -> Result<(), DbErr> {
        // Replace the sample below with your own migration scripts
        manager
            .drop_foreign_key(
                ForeignKeyDropStatement::new()
                    .name("fk_note_mention_user_id")
                    .table(NoteMention::Table)
                    .to_owned(),
            )
            .await?;

        manager
            .drop_foreign_key(
                ForeignKeyDropStatement::new()
                    .name("fk_note_mention_note_id")
                    .table(NoteMention::Table)
                    .to_owned(),
            )
            .await?;

        manager
            .drop_index(
                IndexDropStatement::new()
                    .name("idx_note_mention_unique")
                    .table(NoteMention::Table)
                    .to_owned(),
            )
            .await?;

        manager
            .drop_table(
                TableDropStatement::new()
                    .table(NoteMention::Table)
                    .to_owned(),
            )
            .await?;

        Ok(())
    }
}

#[derive(DeriveIden)]
pub enum NoteMention {
    Table,
    Id,
    NoteId,
    TargetUserId,
}
