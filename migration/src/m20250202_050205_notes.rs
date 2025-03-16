use sea_orm_migration::{
    prelude::{extension::postgres::TypeCreateStatement, *},
    schema::*,
    sea_orm::DbBackend,
};

use crate::common::URL_LENGTH;

#[derive(DeriveMigrationName)]
pub struct Migration;

#[derive(DeriveIden)]
pub enum Note {
    Table,
    Id,
    Url,
    ViewUrl,
    AuthorId,
    Content,
    ContentType,
    CreatedAt,
    InsertedAt,
    UpdatedAt,
    DeletedAt,
    FetchedAt,
    Visibility,
    ReplyToId,
    RenoteOfId,
    Sensitive,
}

#[derive(DeriveIden)]
pub enum Visibility {
    #[sea_orm(iden = "visibility")]
    Enum,
    #[sea_orm(iden = "public")]
    Public,
    #[sea_orm(iden = "unlisted")]
    Unlisted,
    #[sea_orm(iden = "follower")]
    Follower,
    #[sea_orm(iden = "private")]
    Private,
}

#[async_trait::async_trait]
impl MigrationTrait for Migration {
    async fn up(&self, manager: &SchemaManager) -> Result<(), DbErr> {
        // Replace the sample below with your own migration scripts

        let db = manager.get_connection();

        match db.get_database_backend() {
            DbBackend::Postgres => {
                manager
                    .create_type(
                        TypeCreateStatement::new()
                            .as_enum(Visibility::Enum)
                            .values([
                                Visibility::Public,
                                Visibility::Unlisted,
                                Visibility::Follower,
                                Visibility::Private,
                            ])
                            .to_owned(),
                    )
                    .await?
            }
            _ => {}
        }

        manager
            .create_table(
                Table::create()
                    .table(Note::Table)
                    .if_not_exists()
                    .col(uuid(Note::Id).primary_key().not_null())
                    .col(string_len_null(Note::Url, URL_LENGTH))
                    .col(string_len_null(Note::ViewUrl, URL_LENGTH))
                    .col(uuid(Note::AuthorId).not_null())
                    .col(text_null(Note::Content))
                    .col(string_len_null(Note::ContentType, 32))
                    .col(timestamp_with_time_zone(Note::CreatedAt))
                    .col(
                        timestamp_with_time_zone(Note::InsertedAt)
                            .default(Expr::current_timestamp()),
                    )
                    .col(timestamp_with_time_zone_null(Note::UpdatedAt))
                    .col(timestamp_with_time_zone_null(Note::DeletedAt))
                    .col(custom(Note::Visibility, Visibility::Enum))
                    .col(uuid_null(Note::ReplyToId))
                    .col(uuid_null(Note::RenoteOfId))
                    .col(boolean(Note::Sensitive).default(Expr::value(false)))
                    .to_owned(),
            )
            .await
    }

    async fn down(&self, manager: &SchemaManager) -> Result<(), DbErr> {
        // Replace the sample below with your own migration scripts
        manager
            .drop_table(Table::drop().table(Note::Table).to_owned())
            .await
    }
}
