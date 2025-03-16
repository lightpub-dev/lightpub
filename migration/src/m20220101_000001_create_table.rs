use sea_orm_migration::{prelude::*, schema::*};

use crate::common::URL_LENGTH;

#[derive(DeriveMigrationName)]
pub struct Migration;

#[derive(DeriveIden)]
pub enum User {
    Table,
    Id,
    Username,
    Domain,
    Password,
    Nickname,
    Bio,
    Avatar,
    Url,
    Inbox,
    SharedInbox,
    Outbox,
    PrivateKey,
    PublicKey,
    CreatedAt,
    FetchedAt,
    ViewUrl,
    Following,
    Followers,
    AutoFollowAccept,
    AuthExpiredAt,
    IsBot,
    IsAdmin,
    HideFollows,
}

#[async_trait::async_trait]
impl MigrationTrait for Migration {
    async fn up(&self, manager: &SchemaManager) -> Result<(), DbErr> {
        manager
            .create_table(
                Table::create()
                    .table(User::Table)
                    .col(uuid(User::Id).primary_key().not_null())
                    .col(string_len(User::Username, 128))
                    .col(string_len_null(User::Domain, 128))
                    .col(string_len_null(User::Password, 128))
                    .col(string_len(User::Nickname, 255))
                    .col(text(User::Bio))
                    .col(uuid_null(User::Avatar))
                    .col(string_len_null(User::Url, URL_LENGTH))
                    .col(string_len_null(User::Inbox, URL_LENGTH))
                    .col(string_len_null(User::SharedInbox, URL_LENGTH))
                    .col(string_len_null(User::Outbox, URL_LENGTH))
                    .col(text_null(User::PrivateKey))
                    .col(text_null(User::PublicKey))
                    .col(
                        timestamp_with_time_zone(User::CreatedAt)
                            .default(Expr::current_timestamp()),
                    )
                    .col(timestamp_with_time_zone_null(User::FetchedAt))
                    .col(string_len_null(User::ViewUrl, URL_LENGTH))
                    .col(string_len_null(User::Following, URL_LENGTH))
                    .col(string_len_null(User::Followers, URL_LENGTH))
                    .col(boolean(User::AutoFollowAccept).default(Expr::value(true)))
                    .col(timestamp_with_time_zone_null(User::AuthExpiredAt))
                    .col(boolean(User::IsBot).default(Expr::value(false)))
                    .col(boolean(User::IsAdmin).default(Expr::value(false)))
                    .col(boolean(User::HideFollows).default(Expr::value(false)))
                    .to_owned(),
            )
            .await
    }

    async fn down(&self, manager: &SchemaManager) -> Result<(), DbErr> {
        manager
            .drop_table(Table::drop().table(User::Table).to_owned())
            .await
    }
}
