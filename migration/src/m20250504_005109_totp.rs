use sea_orm_migration::{prelude::*, schema::*};

use crate::{
    common::{datetime_6, datetime_6_null},
    m20220101_000001_create_table::User,
};

#[derive(DeriveMigrationName)]
pub struct Migration;

#[async_trait::async_trait]
impl MigrationTrait for Migration {
    async fn up(&self, manager: &SchemaManager) -> Result<(), DbErr> {
        // Replace the sample below with your own migration scripts
        manager
            .create_table(
                Table::create()
                    .table(UserTotp::Table)
                    .col(uuid(UserTotp::Id).primary_key().not_null())
                    .col(string_len(UserTotp::Secret, 128))
                    .col(enumeration(
                        UserTotp::Status,
                        TotpStatus::Enum,
                        [TotpStatus::Setup, TotpStatus::Enabled],
                    ))
                    .col(datetime_6(UserTotp::CreatedAt))
                    .to_owned(),
            )
            .await?;
        manager
            .create_foreign_key(
                ForeignKeyCreateStatement::new()
                    .name("fk_user_totp_user_id")
                    .from(UserTotp::Table, UserTotp::Id)
                    .to(User::Table, User::Id)
                    .on_delete(ForeignKeyAction::Cascade)
                    .on_update(ForeignKeyAction::Cascade)
                    .to_owned(),
            )
            .await?;

        manager
            .create_table(
                Table::create()
                    .table(UserTotpBackup::Table)
                    .col(uuid(UserTotpBackup::Id).primary_key().not_null())
                    .col(string_len(UserTotpBackup::Code, 128))
                    .col(datetime_6(UserTotpBackup::CreatedAt))
                    .col(datetime_6_null(UserTotpBackup::UsedAt))
                    .to_owned(),
            )
            .await?;
        manager
            .create_foreign_key(
                ForeignKeyCreateStatement::new()
                    .name("fk_user_totp_backup_user_id")
                    .from(UserTotpBackup::Table, UserTotpBackup::Id)
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
            .drop_foreign_key(
                ForeignKeyDropStatement::new()
                    .table(UserTotpBackup::Table)
                    .name("fk_user_totp_backup_user_id")
                    .to_owned(),
            )
            .await?;
        manager
            .drop_table(Table::drop().table(UserTotpBackup::Table).to_owned())
            .await?;

        manager
            .drop_foreign_key(
                ForeignKeyDropStatement::new()
                    .table(UserTotp::Table)
                    .name("fk_user_totp_user_id")
                    .to_owned(),
            )
            .await?;
        manager
            .drop_table(Table::drop().table(UserTotp::Table).to_owned())
            .await?;

        Ok(())
    }
}

#[derive(DeriveIden)]
enum UserTotp {
    Table,
    Id,
    Secret,
    Status,
    CreatedAt,
}

#[derive(DeriveIden)]
pub enum TotpStatus {
    #[sea_orm(iden = "totp_status")]
    Enum,
    #[sea_orm(iden = "setup")]
    Setup,
    #[sea_orm(iden = "enabled")]
    Enabled,
}

#[derive(DeriveIden)]
enum UserTotpBackup {
    Table,
    Id,
    Code,
    CreatedAt,
    UsedAt,
}
