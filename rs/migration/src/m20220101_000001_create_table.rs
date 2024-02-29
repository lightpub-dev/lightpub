use sea_orm_migration::prelude::*;

use crate::ident::User;

#[derive(DeriveMigrationName)]
pub struct Migration;

#[async_trait::async_trait]
impl MigrationTrait for Migration {
    async fn up(&self, manager: &SchemaManager) -> Result<(), DbErr> {
        manager
            .create_table(
                Table::create()
                    .table(User::Table)
                    .if_not_exists()
                    .col(ColumnDef::new(User::Id).uuid().not_null().primary_key())
                    .col(ColumnDef::new(User::Username).string_len(64).not_null())
                    .col(ColumnDef::new(User::Host).string_len(128).not_null())
                    .col(ColumnDef::new(User::Bpassword).string_len(60).null())
                    .col(ColumnDef::new(User::Nickname).string_len(255).not_null())
                    .col(ColumnDef::new(User::Bio).text().default(Value::from("")))
                    .col(ColumnDef::new(User::Uri).string_len(512).null())
                    .col(
                        ColumnDef::new(User::CreatedAt)
                            .date_time()
                            .not_null()
                            .default(Expr::current_timestamp()),
                    )
                    .col(ColumnDef::new(User::DeletedAt).timestamp().null())
                    .to_owned(),
            )
            .await
    }

    async fn down(&self, manager: &SchemaManager) -> Result<(), DbErr> {
        manager
            .drop_table(Table::drop().table(User::Table).if_exists().to_owned())
            .await
    }
}
