use sea_orm_migration::{prelude::*, schema::*};

use crate::m20250202_050205_notes::Note;

#[derive(DeriveMigrationName)]
pub struct Migration;

#[async_trait::async_trait]
impl MigrationTrait for Migration {
    async fn up(&self, manager: &SchemaManager) -> Result<(), DbErr> {
        // Replace the sample below with your own migration scripts
        manager
            .create_table(
                TableCreateStatement::new()
                    .table(Tag::Table)
                    .col(pk_auto(Tag::Id))
                    .col(string_len(Tag::Name, 256))
                    .to_owned(),
            )
            .await?;

        manager
            .create_table(
                TableCreateStatement::new()
                    .table(NoteTag::Table)
                    .col(pk_auto(NoteTag::Id))
                    .col(uuid(NoteTag::NoteId))
                    .col(integer(NoteTag::TagId))
                    .to_owned(),
            )
            .await?;

        manager
            .create_foreign_key(
                ForeignKeyCreateStatement::new()
                    .name("fk_note_tag_note_id")
                    .from(NoteTag::Table, NoteTag::NoteId)
                    .to(Note::Table, Note::Id)
                    .on_delete(ForeignKeyAction::Cascade)
                    .on_update(ForeignKeyAction::Cascade)
                    .to_owned(),
            )
            .await?;

        manager
            .create_foreign_key(
                ForeignKeyCreateStatement::new()
                    .name("fk_note_tag_tag_id")
                    .from(NoteTag::Table, NoteTag::TagId)
                    .to(Tag::Table, Tag::Id)
                    .on_delete(ForeignKeyAction::Cascade)
                    .on_update(ForeignKeyAction::Cascade)
                    .to_owned(),
            )
            .await?;

        manager
            .create_index(
                IndexCreateStatement::new()
                    .name("idx_note_tag_unique")
                    .table(NoteTag::Table)
                    .col(NoteTag::NoteId)
                    .col(NoteTag::TagId)
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
                    .name("fk_note_tag_tag_id")
                    .table(NoteTag::Table)
                    .to_owned(),
            )
            .await?;

        manager
            .drop_foreign_key(
                ForeignKeyDropStatement::new()
                    .name("fk_note_tag_note_id")
                    .table(NoteTag::Table)
                    .to_owned(),
            )
            .await?;

        manager
            .drop_index(
                IndexDropStatement::new()
                    .name("idx_note_tag_unique")
                    .table(NoteTag::Table)
                    .to_owned(),
            )
            .await?;

        manager
            .drop_table(TableDropStatement::new().table(NoteTag::Table).to_owned())
            .await?;

        manager
            .drop_table(TableDropStatement::new().table(Tag::Table).to_owned())
            .await?;

        Ok(())
    }
}

#[derive(DeriveIden)]
enum NoteTag {
    Table,
    Id,
    NoteId,
    TagId,
}

#[derive(DeriveIden)]
enum Tag {
    Table,
    Id,
    Name,
}
