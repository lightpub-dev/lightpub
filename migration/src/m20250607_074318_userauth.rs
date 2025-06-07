use crate::m20220101_000001_create_table::User;
use sea_orm_migration::{prelude::*, schema::*};

#[derive(DeriveMigrationName)]
pub struct Migration;

#[async_trait::async_trait]
impl MigrationTrait for Migration {
    async fn up(&self, manager: &SchemaManager) -> Result<(), DbErr> {
        manager
            .create_table(
                TableCreateStatement::new()
                    .table(UserAuth::Table)
                    .col(uuid(UserAuth::UserId).primary_key().not_null())
                    .col(string_len(UserAuth::PasswordHash, 128))
                    .foreign_key(
                        ForeignKeyCreateStatement::new()
                            .name("fk_user_auth_user_id")
                            .from(UserAuth::Table, UserAuth::UserId)
                            .to(User::Table, User::Id)
                            .on_delete(ForeignKeyAction::Cascade)
                            .on_update(ForeignKeyAction::Cascade),
                    )
                    .to_owned(),
            )
            .await?;

        let db = manager.get_connection();
        db.execute_unprepared(
            r#"
        INSERT INTO user_auth (user_id, password_hash)
        SELECT id, password FROM user
        WHERE password IS NOT NULL;
        "#,
        )
        .await?;

        manager
            .alter_table(
                TableAlterStatement::new()
                    .table(User::Table)
                    .drop_column(User::Password)
                    .to_owned(),
            )
            .await?;

        Ok(())
    }

    async fn down(&self, manager: &SchemaManager) -> Result<(), DbErr> {
        manager
            .alter_table(
                TableAlterStatement::new()
                    .table(User::Table)
                    .add_column(string_len_null(User::Password, 128))
                    .to_owned(),
            )
            .await?;

        let db = manager.get_connection();
        db.execute_unprepared(
            r#"
        UPDATE user
        JOIN user_auth ON user.id = user_auth.user_id
        SET user.password = user_auth.password_hash;

        "#,
        )
        .await?;

        manager
            .drop_foreign_key(
                ForeignKeyDropStatement::new()
                    .name("fk_user_auth_user_id")
                    .table(UserAuth::Table)
                    .to_owned(),
            )
            .await?;

        manager
            .drop_table(TableDropStatement::new().table(UserAuth::Table).to_owned())
            .await?;

        Ok(())
    }
}

#[derive(DeriveIden)]
enum UserAuth {
    Table,
    UserId,
    PasswordHash,
}
