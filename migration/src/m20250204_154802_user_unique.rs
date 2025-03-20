use sea_orm_migration::prelude::*;

use crate::m20220101_000001_create_table::User;

#[derive(DeriveMigrationName)]
pub struct Migration;

#[async_trait::async_trait]
impl MigrationTrait for Migration {
    async fn up(&self, manager: &SchemaManager) -> Result<(), DbErr> {
        manager
            .alter_table(
                TableAlterStatement::new()
                    .table(User::Table)
                    .modify_column(
                        ColumnDef::new(User::Domain)
                            .not_null()
                            .string_len(128)
                            .to_owned(),
                    )
                    .to_owned(),
            )
            .await?;

        manager
            .create_index(
                IndexCreateStatement::new()
                    .table(User::Table)
                    .name("user_unique_username")
                    .col(User::Username)
                    .col(User::Domain)
                    .unique()
                    .to_owned(),
            )
            .await
    }

    async fn down(&self, manager: &SchemaManager) -> Result<(), DbErr> {
        // Replace the sample below with your own migration scripts
        manager
            .drop_index(
                IndexDropStatement::new()
                    .table(User::Table)
                    .name("user_unique_username")
                    .to_owned(),
            )
            .await?;

        manager
            .alter_table(
                TableAlterStatement::new()
                    .table(User::Table)
                    .modify_column(
                        ColumnDef::new(User::Domain)
                            .null()
                            .string_len(128)
                            .to_owned(),
                    )
                    .to_owned(),
            )
            .await
    }
}
