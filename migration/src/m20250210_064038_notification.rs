use sea_orm_migration::{prelude::*, schema::*};

use crate::common::{current_timestamp_6, datetime_6, datetime_6_null};

#[derive(DeriveMigrationName)]
pub struct Migration;

#[async_trait::async_trait]
impl MigrationTrait for Migration {
    async fn up(&self, manager: &SchemaManager) -> Result<(), DbErr> {
        // Replace the sample below with your own migration scripts
        manager
            .create_table(
                TableCreateStatement::new()
                    .table(Notification::Table)
                    .col(pk_auto(Notification::Id))
                    .col(uuid(Notification::UserId).not_null())
                    .col(json(Notification::Body).not_null())
                    .col(
                        datetime_6(Notification::CreatedAt)
                            .not_null()
                            .default(current_timestamp_6()),
                    )
                    .col(datetime_6_null(Notification::ReadAt))
                    .to_owned(),
            )
            .await
    }

    async fn down(&self, manager: &SchemaManager) -> Result<(), DbErr> {
        // Replace the sample below with your own migration scripts
        manager
            .drop_table(
                TableDropStatement::new()
                    .table(Notification::Table)
                    .to_owned(),
            )
            .await
    }
}

#[derive(DeriveIden)]
pub enum Notification {
    Table,
    Id,
    UserId,
    Body,
    CreatedAt,
    ReadAt,
}
