use sea_orm_migration::prelude::*;

use crate::ident::{User, UserFollow};

#[derive(DeriveMigrationName)]
pub struct Migration;

#[async_trait::async_trait]
impl MigrationTrait for Migration {
    async fn up(&self, manager: &SchemaManager) -> Result<(), DbErr> {
        manager
            .create_table(
                Table::create()
                    .table(UserFollow::Table)
                    .if_not_exists()
                    .col(
                        ColumnDef::new(UserFollow::Id)
                            .integer()
                            .not_null()
                            .auto_increment()
                            .primary_key(),
                    )
                    .col(ColumnDef::new(UserFollow::FollowerId).uuid().not_null())
                    .col(ColumnDef::new(UserFollow::FolloweeId).uuid().not_null())
                    .col(
                        ColumnDef::new(UserFollow::CreatedAt)
                            .date_time()
                            .not_null()
                            .default(Expr::current_timestamp()),
                    )
                    .to_owned(),
            )
            .await?;

        manager
            .create_foreign_key(
                ForeignKey::create()
                    .name("fk_userfollow_follower")
                    .from(UserFollow::Table, UserFollow::FollowerId)
                    .to(User::Table, User::Id)
                    .on_delete(ForeignKeyAction::Cascade)
                    .to_owned(),
            )
            .await?;

        manager
            .create_foreign_key(
                ForeignKey::create()
                    .name("fk_userfollow_followee")
                    .from(UserFollow::Table, UserFollow::FolloweeId)
                    .to(User::Table, User::Id)
                    .on_delete(ForeignKeyAction::Cascade)
                    .to_owned(),
            )
            .await?;

        Ok(())
    }

    async fn down(&self, manager: &SchemaManager) -> Result<(), DbErr> {
        manager
            .drop_foreign_key(
                ForeignKey::drop()
                    .name("fk_userfollow_followee")
                    .table(UserFollow::Table)
                    .to_owned(),
            )
            .await?;

        manager
            .drop_foreign_key(
                ForeignKey::drop()
                    .name("fk_userfollow_follower")
                    .table(UserFollow::Table)
                    .to_owned(),
            )
            .await?;

        manager
            .drop_table(Table::drop().table(UserFollow::Table).to_owned())
            .await?;

        Ok(())
    }
}
