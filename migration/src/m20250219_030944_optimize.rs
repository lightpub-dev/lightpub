use sea_orm_migration::prelude::*;

use crate::m20250210_064038_notification::Notification;

#[derive(DeriveMigrationName)]
pub struct Migration;

#[async_trait::async_trait]
impl MigrationTrait for Migration {
    async fn up(&self, manager: &SchemaManager) -> Result<(), DbErr> {
        // Replace the sample below with your own migration scripts
        manager
            .create_index(
                IndexCreateStatement::new()
                    .name("idx_notification_read_at")
                    .table(Notification::Table)
                    .col(Notification::UserId)
                    .col(Notification::ReadAt)
                    .to_owned(),
            )
            .await?;

        Ok(())
    }

    async fn down(&self, manager: &SchemaManager) -> Result<(), DbErr> {
        // Replace the sample below with your own migration scripts
        manager
            .drop_index(
                IndexDropStatement::new()
                    .name("idx_notification_read_at")
                    .table(Notification::Table)
                    .to_owned(),
            )
            .await?;

        Ok(())
    }
}
