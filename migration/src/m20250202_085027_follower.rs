use sea_orm_migration::{prelude::*, schema::*};

use crate::common::{current_timestamp_6, datetime_6, URL_LENGTH};

#[derive(DeriveMigrationName)]
pub struct Migration;

#[derive(DeriveIden)]
pub enum UserFollow {
    Table,
    Id,
    FollowerId,
    FollowedId,
    Pending,
    Url,
    CreatedAt,
}

#[async_trait::async_trait]
impl MigrationTrait for Migration {
    async fn up(&self, manager: &SchemaManager) -> Result<(), DbErr> {
        // Replace the sample below with your own migration scripts
        manager
            .create_table(
                Table::create()
                    .table(UserFollow::Table)
                    .if_not_exists()
                    .col(pk_auto(UserFollow::Id))
                    .col(uuid(UserFollow::FollowerId).not_null())
                    .col(uuid(UserFollow::FollowedId).not_null())
                    .col(boolean(UserFollow::Pending).not_null())
                    .col(string_len_null(UserFollow::Url, URL_LENGTH))
                    .col(datetime_6(UserFollow::CreatedAt).default(current_timestamp_6()))
                    .to_owned(),
            )
            .await?;

        manager
            .create_index(
                Index::create()
                    .name("user_follow_unique")
                    .table(UserFollow::Table)
                    .col(UserFollow::FollowerId)
                    .col(UserFollow::FollowedId)
                    .unique()
                    .to_owned(),
            )
            .await
    }

    async fn down(&self, manager: &SchemaManager) -> Result<(), DbErr> {
        // Replace the sample below with your own migration scripts
        manager
            .drop_index(
                Index::drop()
                    .table(UserFollow::Table)
                    .name("user_follow_unique")
                    .to_owned(),
            )
            .await?;

        manager
            .drop_table(Table::drop().table(UserFollow::Table).to_owned())
            .await
    }
}
