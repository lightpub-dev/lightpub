use sea_orm_migration::prelude::*;

use crate::{m20220101_000001_create_table::User, m20250211_132721_uploads::Upload};

#[derive(DeriveMigrationName)]
pub struct Migration;

#[async_trait::async_trait]
impl MigrationTrait for Migration {
    async fn up(&self, manager: &SchemaManager) -> Result<(), DbErr> {
        // Replace the sample below with your own migration scripts
        manager
            .create_foreign_key(
                ForeignKeyCreateStatement::new()
                    .name("fk_user_avatar")
                    .from(User::Table, User::Avatar)
                    .to(Upload::Table, Upload::Id)
                    .on_delete(ForeignKeyAction::SetNull)
                    .on_update(ForeignKeyAction::Cascade)
                    .to_owned(),
            )
            .await
    }

    async fn down(&self, manager: &SchemaManager) -> Result<(), DbErr> {
        // Replace the sample below with your own migration scripts
        manager
            .drop_foreign_key(
                ForeignKeyDropStatement::new()
                    .name("fk_user_avatar")
                    .table(User::Table)
                    .to_owned(),
            )
            .await
    }
}
