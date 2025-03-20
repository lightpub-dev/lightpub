use sea_orm_migration::prelude::*;

use crate::{m20220101_000001_create_table::User, m20250202_085027_follower::UserFollow};

#[derive(DeriveMigrationName)]
pub struct Migration;

#[async_trait::async_trait]
impl MigrationTrait for Migration {
    async fn up(&self, manager: &SchemaManager) -> Result<(), DbErr> {
        // Replace the sample below with your own migration scripts
        manager
            .create_foreign_key(
                ForeignKeyCreateStatement::new()
                    .name("fk_user_follow_follower_id")
                    .from(UserFollow::Table, UserFollow::FollowerId)
                    .to(User::Table, User::Id)
                    .on_update(ForeignKeyAction::Cascade)
                    .on_delete(ForeignKeyAction::Cascade)
                    .to_owned(),
            )
            .await?;

        manager
            .create_foreign_key(
                ForeignKeyCreateStatement::new()
                    .name("fk_user_follow_followed_id")
                    .from(UserFollow::Table, UserFollow::FollowedId)
                    .to(User::Table, User::Id)
                    .on_update(ForeignKeyAction::Cascade)
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
                    .table(UserFollow::Table)
                    .name("fk_user_follow_followed_id")
                    .to_owned(),
            )
            .await?;

        manager
            .drop_foreign_key(
                ForeignKeyDropStatement::new()
                    .table(UserFollow::Table)
                    .name("fk_user_follow_follower_id")
                    .to_owned(),
            )
            .await
    }
}
