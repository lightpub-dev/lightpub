use sea_orm_migration::prelude::*;

use crate::ident::{Post, PostHashtag, PostLike, PostMention, PostReaction, Reaction, User};

#[derive(DeriveMigrationName)]
pub struct Migration;

#[async_trait::async_trait]
impl MigrationTrait for Migration {
    async fn up(&self, manager: &SchemaManager) -> Result<(), DbErr> {
        // Post
        manager
            .create_table(
                Table::create()
                    .table(Post::Table)
                    .if_not_exists()
                    .col(ColumnDef::new(Post::Id).uuid().not_null().primary_key())
                    .col(ColumnDef::new(Post::PosterId).uuid().not_null())
                    .col(ColumnDef::new(Post::Content).text().null())
                    .col(
                        ColumnDef::new(Post::InsertedAt)
                            .date_time()
                            .not_null()
                            .default(Expr::current_timestamp()),
                    )
                    .col(
                        ColumnDef::new(Post::CreatedAt)
                            .date_time()
                            .not_null()
                            .default(Expr::current_timestamp()),
                    )
                    .col(
                        ColumnDef::new(Post::Privacy)
                            .enumeration(
                                Alias::new("privacy"),
                                [
                                    Alias::new("public"),
                                    Alias::new("unlisted"),
                                    Alias::new("followers"),
                                    Alias::new("private"),
                                ],
                            )
                            .not_null(),
                    )
                    .col(ColumnDef::new(Post::ReplyToId).uuid().null())
                    .col(ColumnDef::new(Post::RepostOfId).uuid().null())
                    .col(ColumnDef::new(Post::DeletedAt).date_time().null())
                    .to_owned(),
            )
            .await?;
        manager
            .create_foreign_key(
                ForeignKey::create()
                    .name("fk_post_poster")
                    .from(Post::Table, Post::PosterId)
                    .to(User::Table, User::Id)
                    .on_delete(ForeignKeyAction::Restrict)
                    .to_owned(),
            )
            .await?;

        // PostHashtag
        manager
            .create_table(
                Table::create()
                    .table(PostHashtag::Table)
                    .col(ColumnDef::new(PostHashtag::PostId).uuid().not_null())
                    .col(ColumnDef::new(PostHashtag::Name).string_len(255).not_null())
                    .to_owned(),
            )
            .await?;
        manager
            .create_foreign_key(
                ForeignKey::create()
                    .name("fk_hashtag_post")
                    .from(PostHashtag::Table, PostHashtag::PostId)
                    .to(Post::Table, Post::Id)
                    .on_delete(ForeignKeyAction::Cascade)
                    .to_owned(),
            )
            .await?;
        manager
            .create_index(
                Index::create()
                    .table(PostHashtag::Table)
                    .col(PostHashtag::PostId)
                    .col(PostHashtag::Name)
                    .primary()
                    .to_owned(),
            )
            .await?;

        // PostLike
        manager
            .create_table(
                Table::create()
                    .table(PostLike::Table)
                    .col(ColumnDef::new(PostLike::PostId).uuid().not_null())
                    .col(ColumnDef::new(PostLike::UserId).uuid().not_null())
                    .col(
                        ColumnDef::new(PostLike::CreatedAt)
                            .date_time()
                            .not_null()
                            .default(Expr::current_timestamp()),
                    )
                    .col(ColumnDef::new(PostLike::IsPrivate).boolean().not_null())
                    .to_owned(),
            )
            .await?;
        manager
            .create_foreign_key(
                ForeignKey::create()
                    .name("fk_postlike_post")
                    .from(PostLike::Table, PostLike::PostId)
                    .to(Post::Table, Post::Id)
                    .on_delete(ForeignKeyAction::Cascade)
                    .to_owned(),
            )
            .await?;
        manager
            .create_foreign_key(
                ForeignKey::create()
                    .name("fk_postlike_user")
                    .from(PostLike::Table, PostLike::UserId)
                    .to(User::Table, User::Id)
                    .on_delete(ForeignKeyAction::Cascade)
                    .to_owned(),
            )
            .await?;
        manager
            .create_index(
                Index::create()
                    .table(PostLike::Table)
                    .col(PostLike::PostId)
                    .col(PostLike::UserId)
                    .col(PostLike::IsPrivate)
                    .primary()
                    .to_owned(),
            )
            .await?;

        // PostMention
        manager
            .create_table(
                Table::create()
                    .table(PostMention::Table)
                    .col(ColumnDef::new(PostMention::PostId).uuid().not_null())
                    .col(ColumnDef::new(PostMention::TargetUserId).uuid().not_null())
                    .col(
                        ColumnDef::new(PostMention::CreatedAt)
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
                    .name("fk_postmention_post")
                    .from(PostMention::Table, PostMention::PostId)
                    .to(Post::Table, Post::Id)
                    .on_delete(ForeignKeyAction::Cascade)
                    .to_owned(),
            )
            .await?;
        manager
            .create_foreign_key(
                ForeignKey::create()
                    .name("fk_postmention_user")
                    .from(PostMention::Table, PostMention::TargetUserId)
                    .to(User::Table, User::Id)
                    .on_delete(ForeignKeyAction::Cascade)
                    .to_owned(),
            )
            .await?;
        manager
            .create_index(
                Index::create()
                    .table(PostMention::Table)
                    .col(PostMention::PostId)
                    .col(PostMention::TargetUserId)
                    .primary()
                    .to_owned(),
            )
            .await?;

        // Reaction
        manager
            .create_table(
                Table::create()
                    .table(Reaction::Table)
                    .col(ColumnDef::new(Reaction::Id).uuid().not_null().primary_key())
                    .col(
                        ColumnDef::new(Reaction::Name)
                            .string_len(255)
                            .not_null()
                            .unique_key(),
                    )
                    .col(ColumnDef::new(Reaction::ImageId).uuid().not_null())
                    .col(
                        ColumnDef::new(Reaction::RegisteredAt)
                            .date_time()
                            .not_null()
                            .default(Expr::current_timestamp()),
                    )
                    .to_owned(),
            )
            .await?;

        // PostReaction
        manager
            .create_table(
                Table::create()
                    .table(PostReaction::Table)
                    .col(ColumnDef::new(PostReaction::PostId).uuid().not_null())
                    .col(ColumnDef::new(PostReaction::UserId).uuid().not_null())
                    .col(ColumnDef::new(PostReaction::ReactionId).uuid().not_null())
                    .col(
                        ColumnDef::new(PostReaction::CreatedAt)
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
                    .name("fk_postreaction_post")
                    .from(PostReaction::Table, PostReaction::PostId)
                    .to(Post::Table, Post::Id)
                    .on_delete(ForeignKeyAction::Cascade)
                    .to_owned(),
            )
            .await?;
        manager
            .create_foreign_key(
                ForeignKey::create()
                    .name("fk_postreaction_user")
                    .from(PostReaction::Table, PostReaction::UserId)
                    .to(User::Table, User::Id)
                    .on_delete(ForeignKeyAction::Cascade)
                    .to_owned(),
            )
            .await?;
        manager
            .create_foreign_key(
                ForeignKey::create()
                    .name("fk_postreaction_reaction")
                    .from(PostReaction::Table, PostReaction::ReactionId)
                    .to(Reaction::Table, Reaction::Id)
                    .on_delete(ForeignKeyAction::Cascade)
                    .to_owned(),
            )
            .await?;
        manager
            .create_index(
                Index::create()
                    .table(PostReaction::Table)
                    .col(PostReaction::PostId)
                    .col(PostReaction::UserId)
                    .col(PostReaction::ReactionId)
                    .primary()
                    .to_owned(),
            )
            .await?;

        Ok(())
    }

    async fn down(&self, manager: &SchemaManager) -> Result<(), DbErr> {
        manager
            .drop_foreign_key(
                ForeignKey::drop()
                    .table(PostReaction::Table)
                    .name("fk_postreaction_reaction")
                    .to_owned(),
            )
            .await?;
        manager
            .drop_foreign_key(
                ForeignKey::drop()
                    .table(PostReaction::Table)
                    .name("fk_postreaction_user")
                    .to_owned(),
            )
            .await?;
        manager
            .drop_foreign_key(
                ForeignKey::drop()
                    .table(PostReaction::Table)
                    .name("fk_postreaction_post")
                    .to_owned(),
            )
            .await?;
        manager
            .drop_index(
                Index::drop()
                    .table(PostReaction::Table)
                    .name("idx_postreaction_unique")
                    .to_owned(),
            )
            .await?;
        manager
            .drop_table(Table::drop().table(PostReaction::Table).to_owned())
            .await?;

        manager
            .drop_table(Table::drop().table(Reaction::Table).to_owned())
            .await?;

        manager
            .drop_foreign_key(
                ForeignKey::drop()
                    .table(PostMention::Table)
                    .name("fk_postmention_user")
                    .to_owned(),
            )
            .await?;
        manager
            .drop_foreign_key(
                ForeignKey::drop()
                    .table(PostMention::Table)
                    .name("fk_postmention_post")
                    .to_owned(),
            )
            .await?;
        manager
            .drop_index(
                Index::drop()
                    .table(PostMention::Table)
                    .name("idx_postmention_unique")
                    .to_owned(),
            )
            .await?;
        manager
            .drop_table(Table::drop().table(PostMention::Table).to_owned())
            .await?;

        manager
            .drop_foreign_key(
                ForeignKey::drop()
                    .table(PostLike::Table)
                    .name("fk_postlike_user")
                    .to_owned(),
            )
            .await?;
        manager
            .drop_foreign_key(
                ForeignKey::drop()
                    .table(PostLike::Table)
                    .name("fk_postlike_post")
                    .to_owned(),
            )
            .await?;
        manager
            .drop_index(
                Index::drop()
                    .table(PostLike::Table)
                    .name("idx_postlike_unique")
                    .to_owned(),
            )
            .await?;
        manager
            .drop_table(Table::drop().table(PostLike::Table).to_owned())
            .await?;

        manager
            .drop_foreign_key(
                ForeignKey::drop()
                    .table(PostHashtag::Table)
                    .name("fk_hashtag_post")
                    .to_owned(),
            )
            .await?;
        manager
            .drop_index(
                Index::drop()
                    .table(PostHashtag::Table)
                    .name("idx_hashtag_unique")
                    .to_owned(),
            )
            .await?;
        manager
            .drop_table(Table::drop().table(PostHashtag::Table).to_owned())
            .await?;

        manager
            .drop_foreign_key(
                ForeignKey::drop()
                    .table(Post::Table)
                    .name("fk_post_poster")
                    .to_owned(),
            )
            .await?;
        manager
            .drop_table(Table::drop().table(Post::Table).to_owned())
            .await?;

        Ok(())
    }
}
