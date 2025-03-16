use sea_orm_migration::{prelude::*, schema::*};

use crate::{common::URL_LENGTH, m20250202_050205_notes::Note};

#[derive(DeriveMigrationName)]
pub struct Migration;

#[async_trait::async_trait]
impl MigrationTrait for Migration {
    async fn up(&self, manager: &SchemaManager) -> Result<(), DbErr> {
        // Replace the sample below with your own migration scripts
        manager
            .create_table(
                TableCreateStatement::new()
                    .table(Upload::Table)
                    .col(uuid(Upload::Id).not_null().primary_key())
                    .col(string_len_null(Upload::Filename, 64))
                    .col(string_len_null(Upload::Url, URL_LENGTH))
                    .to_owned(),
            )
            .await?;

        manager
            .create_table(
                TableCreateStatement::new()
                    .table(NoteUpload::Table)
                    .col(pk_auto(NoteUpload::Id))
                    .col(uuid(NoteUpload::NoteId))
                    .col(uuid(NoteUpload::UploadId))
                    .to_owned(),
            )
            .await?;

        manager
            .create_foreign_key(
                ForeignKeyCreateStatement::new()
                    .name("fk_note_upload_note_id")
                    .from(NoteUpload::Table, NoteUpload::NoteId)
                    .to(Note::Table, Note::Id)
                    .on_delete(ForeignKeyAction::Cascade)
                    .on_update(ForeignKeyAction::Cascade)
                    .to_owned(),
            )
            .await?;

        manager
            .create_foreign_key(
                ForeignKeyCreateStatement::new()
                    .name("fk_note_upload_upload_id")
                    .from(NoteUpload::Table, NoteUpload::UploadId)
                    .to(Upload::Table, Upload::Id)
                    .on_delete(ForeignKeyAction::Cascade)
                    .on_update(ForeignKeyAction::Cascade)
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
                    .name("fk_note_upload_note_id")
                    .table(NoteUpload::Table)
                    .to_owned(),
            )
            .await?;

        manager
            .drop_foreign_key(
                ForeignKeyDropStatement::new()
                    .name("fk_note_upload_upload_id")
                    .table(NoteUpload::Table)
                    .to_owned(),
            )
            .await?;

        manager
            .drop_table(
                TableDropStatement::new()
                    .table(NoteUpload::Table)
                    .to_owned(),
            )
            .await?;

        manager
            .drop_table(TableDropStatement::new().table(Upload::Table).to_owned())
            .await?;

        Ok(())
    }
}

#[derive(DeriveIden)]
pub enum Upload {
    Table,
    Id,
    Filename,
    Url,
    MimeType,
}

#[derive(DeriveIden)]
enum NoteUpload {
    Table,
    Id,
    NoteId,
    UploadId,
}
