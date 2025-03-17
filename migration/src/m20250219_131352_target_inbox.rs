use sea_orm_migration::prelude::*;

#[derive(DeriveMigrationName)]
pub struct Migration;

#[async_trait::async_trait]
impl MigrationTrait for Migration {
    async fn up(&self, manager: &SchemaManager) -> Result<(), DbErr> {
        // Replace the sample below with your own migration scripts
        let db = manager.get_connection();

        db.execute_unprepared(
            r#"
ALTER TABLE `user`
ADD COLUMN `preferred_inbox` VARCHAR(512) AS (COALESCE(`shared_inbox`, `inbox`)) PERSISTENT;
        "#,
        )
        .await?;

        Ok(())
    }

    async fn down(&self, manager: &SchemaManager) -> Result<(), DbErr> {
        // Replace the sample below with your own migration scripts
        let db = manager.get_connection();

        db.execute_unprepared(
            r#"
        ALTER TABLE `user` DROP COLUMN `preferred_inbox`;
        "#,
        )
        .await?;

        Ok(())
    }
}
