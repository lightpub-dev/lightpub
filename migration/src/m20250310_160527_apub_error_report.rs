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
                    .table(ApubErrorReport::Table)
                    .col(pk_auto(ApubErrorReport::Id))
                    .col(text(ApubErrorReport::Activity).not_null())
                    .col(text(ApubErrorReport::ErrorMsg).not_null())
                    .col(timestamp_with_time_zone(ApubErrorReport::ReceivedAt).not_null())
                    .to_owned(),
            )
            .await
    }

    async fn down(&self, manager: &SchemaManager) -> Result<(), DbErr> {
        // Replace the sample below with your own migration scripts
        manager
            .drop_table(
                TableDropStatement::new()
                    .table(ApubErrorReport::Table)
                    .to_owned(),
            )
            .await
    }
}

#[derive(DeriveIden)]
enum ApubErrorReport {
    Table,
    Id,
    Activity,
    ErrorMsg,
    ReceivedAt,
}
