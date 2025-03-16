use sea_orm_migration::prelude::*;

use crate::{m20220101_000001_create_table::User, m20250202_050205_notes::Note};

#[derive(DeriveMigrationName)]
pub struct Migration;

#[async_trait::async_trait]
impl MigrationTrait for Migration {
    async fn up(&self, manager: &SchemaManager) -> Result<(), DbErr> {
        // Replace the sample below with your own migration scripts
        manager
            .create_foreign_key(
                ForeignKeyCreateStatement::new()
                    .name("fk_note_author_id")
                    .from(Note::Table, Note::AuthorId)
                    .to(User::Table, User::Id)
                    .on_update(ForeignKeyAction::Cascade)
                    .on_delete(ForeignKeyAction::Cascade)
                    .to_owned(),
            )
            .await
    }

    async fn down(&self, manager: &SchemaManager) -> Result<(), DbErr> {
        // Replace the sample below with your own migration scripts
        manager
            .drop_foreign_key(
                ForeignKeyDropStatement::new()
                    .table(Note::Table)
                    .name("fk_note_author_id")
                    .to_owned(),
            )
            .await
    }
}
