use sea_orm_migration::prelude::*;

use crate::{m20220101_000001_create_table::User, m20250210_064038_notification::Notification};

#[derive(DeriveMigrationName)]
pub struct Migration;

#[async_trait::async_trait]
impl MigrationTrait for Migration {
    async fn up(&self, manager: &SchemaManager) -> Result<(), DbErr> {
        // Replace the sample below with your own migration scripts
        manager
            .create_foreign_key(
                ForeignKeyCreateStatement::new()
                    .name("fk_notification_user_id")
                    .from(Notification::Table, Notification::UserId)
                    .to(User::Table, User::Id)
                    .on_delete(ForeignKeyAction::Cascade)
                    .to_owned(),
            )
            .await
    }

    async fn down(&self, manager: &SchemaManager) -> Result<(), DbErr> {
        // Replace the sample below with your own migration scripts
        manager
            .drop_foreign_key(
                ForeignKeyDropStatement::new()
                    .name("fk_notification_user_id")
                    .table(Notification::Table)
                    .to_owned(),
            )
            .await
    }
}
