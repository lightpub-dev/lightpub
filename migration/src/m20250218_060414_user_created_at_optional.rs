use sea_orm_migration::prelude::*;

use crate::{
    common::{current_timestamp_6, datetime_6, datetime_6_null},
    m20220101_000001_create_table::User,
};

#[derive(DeriveMigrationName)]
pub struct Migration;

#[async_trait::async_trait]
impl MigrationTrait for Migration {
    async fn up(&self, manager: &SchemaManager) -> Result<(), DbErr> {
        // Replace the sample below with your own migration scripts
        manager
            .alter_table(
                TableAlterStatement::new()
                    .table(User::Table)
                    .modify_column(datetime_6_null(User::CreatedAt))
                    .to_owned(),
            )
            .await?;

        Ok(())
    }

    async fn down(&self, manager: &SchemaManager) -> Result<(), DbErr> {
        // Replace the sample below with your own migration scripts
        manager
            .alter_table(
                TableAlterStatement::new()
                    .table(User::Table)
                    .modify_column(datetime_6(User::CreatedAt).default(current_timestamp_6()))
                    .to_owned(),
            )
            .await?;

        Ok(())
    }
}
