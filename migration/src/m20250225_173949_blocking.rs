use sea_orm_migration::{prelude::*, schema::*};

use crate::m20220101_000001_create_table::User;

#[derive(DeriveMigrationName)]
pub struct Migration;

#[async_trait::async_trait]
impl MigrationTrait for Migration {
    async fn up(&self, manager: &SchemaManager) -> Result<(), DbErr> {
        // Replace the sample below with your own migration scripts
        manager
            .create_table(
                TableCreateStatement::new()
                    .table(UserBlock::Table)
                    .col(pk_auto(UserBlock::Id))
                    .col(uuid(UserBlock::BlockerId))
                    .col(uuid(UserBlock::BlockedId))
                    .col(
                        timestamp_with_time_zone(UserBlock::BlockedAt)
                            .default(Expr::current_timestamp()),
                    )
                    .to_owned(),
            )
            .await?;

        manager
            .create_foreign_key(
                ForeignKeyCreateStatement::new()
                    .name("fk_user_block_blocker_id")
                    .from(UserBlock::Table, UserBlock::BlockerId)
                    .to(User::Table, User::Id)
                    .on_delete(ForeignKeyAction::Cascade)
                    .on_update(ForeignKeyAction::Cascade)
                    .to_owned(),
            )
            .await?;

        manager
            .create_foreign_key(
                ForeignKeyCreateStatement::new()
                    .name("fk_user_block_blocked_id")
                    .from(UserBlock::Table, UserBlock::BlockedId)
                    .to(User::Table, User::Id)
                    .on_delete(ForeignKeyAction::Cascade)
                    .on_update(ForeignKeyAction::Cascade)
                    .to_owned(),
            )
            .await?;

        Ok(())
    }

    async fn down(&self, manager: &SchemaManager) -> Result<(), DbErr> {
        // Replace the sample below with your own migration scripts
        manager
            .drop_table(
                TableDropStatement::new()
                    .table(UserBlock::Table)
                    .cascade()
                    .to_owned(),
            )
            .await?;

        Ok(())
    }
}

#[derive(DeriveIden)]
enum UserBlock {
    Table,
    Id,
    BlockerId,
    BlockedId,
    BlockedAt,
}
