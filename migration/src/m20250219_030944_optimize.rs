use sea_orm_migration::prelude::*;

use crate::{
    m20250202_085027_follower::UserFollow, m20250210_064038_notification::Notification,
    m20250211_025745_mention::NoteMention,
};

#[derive(DeriveMigrationName)]
pub struct Migration;

#[async_trait::async_trait]
impl MigrationTrait for Migration {
    async fn up(&self, manager: &SchemaManager) -> Result<(), DbErr> {
        // Replace the sample below with your own migration scripts
        manager
            .create_index(
                IndexCreateStatement::new()
                    .name("idx_user_follow_followed_id")
                    .table(UserFollow::Table)
                    .col(UserFollow::FollowedId)
                    .to_owned(),
            )
            .await?;

        manager
            .create_index(
                IndexCreateStatement::new()
                    .name("idx_note_mention_target_user_id")
                    .table(NoteMention::Table)
                    .col(NoteMention::TargetUserId)
                    .to_owned(),
            )
            .await?;

        manager
            .create_index(
                IndexCreateStatement::new()
                    .name("idx_notification_user_id")
                    .table(Notification::Table)
                    .col(Notification::UserId)
                    .col((Notification::CreatedAt, IndexOrder::Desc))
                    .to_owned(),
            )
            .await?;

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
                    .name("idx_user_follow_followed_id")
                    .table(UserFollow::Table)
                    .to_owned(),
            )
            .await?;

        manager
            .drop_index(
                IndexDropStatement::new()
                    .name("idx_note_mention_target_user_id")
                    .table(NoteMention::Table)
                    .to_owned(),
            )
            .await?;

        manager
            .drop_index(
                IndexDropStatement::new()
                    .name("idx_notification_user_id")
                    .table(Notification::Table)
                    .to_owned(),
            )
            .await?;

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
