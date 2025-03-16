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
                Table::create()
                    .table(NoteLike::Table)
                    .if_not_exists()
                    .col(pk_auto(NoteLike::Id))
                    .col(uuid(NoteLike::NoteId))
                    .col(uuid(NoteLike::UserId))
                    .col(boolean(NoteLike::IsPrivate))
                    .col(
                        timestamp_with_time_zone(NoteLike::CreatedAt)
                            .default(Expr::current_timestamp()),
                    )
                    .to_owned(),
            )
            .await?;

        manager
            .create_foreign_key(
                ForeignKeyCreateStatement::new()
                    .name("fk_note_like_note_id")
                    .from(NoteLike::Table, NoteLike::NoteId)
                    .to(Note::Table, Note::Id)
                    .on_delete(ForeignKeyAction::Cascade)
                    .to_owned(),
            )
            .await?;

        manager
            .create_foreign_key(
                ForeignKeyCreateStatement::new()
                    .name("fk_note_like_user_id")
                    .from(NoteLike::Table, NoteLike::UserId)
                    .to(User::Table, User::Id)
                    .on_delete(ForeignKeyAction::Cascade)
                    .to_owned(),
            )
            .await?;

        manager
            .create_index(
                Index::create()
                    .name("idx_note_like_unique")
                    .table(NoteLike::Table)
                    .col(NoteLike::NoteId)
                    .col(NoteLike::UserId)
                    .col(NoteLike::IsPrivate)
                    .unique()
                    .to_owned(),
            )
            .await
    }

    async fn down(&self, manager: &SchemaManager) -> Result<(), DbErr> {
        // Replace the sample below with your own migration scripts
        manager
            .drop_index(
                IndexDropStatement::new()
                    .name("idx_note_like_unique")
                    .table(NoteLike::Table)
                    .to_owned(),
            )
            .await?;

        manager
            .drop_foreign_key(
                ForeignKeyDropStatement::new()
                    .name("fk_note_like_user_id")
                    .table(NoteLike::Table)
                    .to_owned(),
            )
            .await?;

        manager
            .drop_foreign_key(
                ForeignKeyDropStatement::new()
                    .name("fk_note_like_note_id")
                    .table(NoteLike::Table)
                    .to_owned(),
            )
            .await?;

        manager
            .drop_table(TableDropStatement::new().table(NoteLike::Table).to_owned())
            .await
    }
}

#[derive(DeriveIden)]
enum NoteLike {
    Table,
    Id,
    NoteId,
    UserId,
    IsPrivate,
    CreatedAt,
}
