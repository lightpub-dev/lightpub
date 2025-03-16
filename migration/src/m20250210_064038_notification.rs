use sea_orm_migration::{prelude::*, schema::*};

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
                        timestamp_with_time_zone(Notification::CreatedAt)
                            .not_null()
                            .default(Expr::current_timestamp()),
                    )
                    .col(timestamp_with_time_zone_null(Notification::ReadAt))
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
