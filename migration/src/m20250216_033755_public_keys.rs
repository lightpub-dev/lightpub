use sea_orm_migration::prelude::*;
use sea_orm_migration::schema::*;

use crate::{common::URL_LENGTH, m20220101_000001_create_table::User};

#[derive(DeriveMigrationName)]
pub struct Migration;

#[async_trait::async_trait]
impl MigrationTrait for Migration {
    async fn up(&self, manager: &SchemaManager) -> Result<(), DbErr> {
        // Replace the sample below with your own migration scripts
        manager
            .create_table(
                TableCreateStatement::new()
                    .table(RemotePublicKey::Table)
                    .col(pk_auto(RemotePublicKey::Id))
                    .col(uuid(RemotePublicKey::OwnerId))
                    .col(string_len(RemotePublicKey::KeyId, URL_LENGTH))
                    .col(text(RemotePublicKey::PublicKey))
                    .to_owned(),
            )
            .await?;

        manager
            .create_foreign_key(
                ForeignKeyCreateStatement::new()
                    .name("fk_remote_public_key_owner_id")
                    .from(RemotePublicKey::Table, RemotePublicKey::OwnerId)
                    .to(User::Table, User::Id)
                    .on_delete(ForeignKeyAction::Cascade)
                    .on_update(ForeignKeyAction::Cascade)
                    .to_owned(),
            )
            .await?;

        manager
            .create_index(
                IndexCreateStatement::new()
                    .name("idx_remote_public_key_key_id_unique")
                    .table(RemotePublicKey::Table)
                    .col(RemotePublicKey::KeyId)
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
                    .table(RemotePublicKey::Table)
                    .to_owned(),
            )
            .await?;

        manager
            .drop_foreign_key(
                ForeignKeyDropStatement::new()
                    .table(RemotePublicKey::Table)
                    .name("fk_remote_public_key_owner_id")
                    .to_owned(),
            )
            .await?;

        manager
            .drop_index(
                IndexDropStatement::new()
                    .name("idx_remote_public_key_key_id_unique")
                    .table(RemotePublicKey::Table)
                    .to_owned(),
            )
            .await?;

        Ok(())
    }
}

#[derive(DeriveIden)]
enum RemotePublicKey {
    Table,
    Id,
    OwnerId,
    KeyId,
    PublicKey,
}
