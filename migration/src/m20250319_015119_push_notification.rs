use sea_orm_migration::{prelude::*, schema::*};

use crate::common::{current_timestamp_6, datetime_6, URL_LENGTH};

#[derive(DeriveMigrationName)]
pub struct Migration;

#[async_trait::async_trait]
impl MigrationTrait for Migration {
    async fn up(&self, manager: &SchemaManager) -> Result<(), DbErr> {
        manager
            .create_table(
                TableCreateStatement::new()
                    .table(PushNotification::Table)
                    .col(pk_auto(PushNotification::Id))
                    .col(uuid(PushNotification::UserId))
                    .col(string_len(PushNotification::Endpoint, URL_LENGTH))
                    .col(text(PushNotification::P256dh))
                    .col(text(PushNotification::Auth))
                    .col(datetime_6(PushNotification::CreatedAt).default(current_timestamp_6()))
                    .to_owned(),
            )
            .await?;

        Ok(())
    }

    async fn down(&self, manager: &SchemaManager) -> Result<(), DbErr> {
        manager
            .drop_table(
                TableDropStatement::new()
                    .table(PushNotification::Table)
                    .to_owned(),
            )
            .await?;

        Ok(())
    }
}

#[derive(DeriveIden)]
enum PushNotification {
    Table,
    Id,
    UserId,
    Endpoint,
    P256dh,
    Auth,
    CreatedAt,
}
