use sea_orm_migration::prelude::*;

#[derive(DeriveIden)]
pub enum User {
    Table,
    Id,
    Username,
    Host,
    Bpassword,
    Nickname,
    Bio,
    Uri,
    CreatedAt,
    DeletedAt,
}

#[derive(DeriveIden)]
pub enum UserFollow {
    Table,
    Id,
    FollowerId,
    FolloweeId,
    CreatedAt,
}

#[derive(DeriveIden)]
pub enum UserToken {
    Table,
    Token,
    UserId,
    CreatedAt,
    LastUsedAt,
}

#[derive(DeriveIden)]
pub enum Post {
    Table,
    Id,
    PosterId,
    Content,
    InsertedAt,
    CreatedAt,
    Privacy,
    ReplyToId,
    RepostOfId,
    DeletedAt,
}

#[derive(DeriveIden)]
pub enum PostHashtag {
    Table,
    PostId,
    Name,
}

#[derive(DeriveIden)]
pub enum PostLike {
    Table,
    PostId,
    UserId,
    CreatedAt,
    IsPrivate,
}

#[derive(DeriveIden)]
pub enum PostMention {
    Table,
    PostId,
    TargetUserId,
    CreatedAt,
}

#[derive(DeriveIden)]
pub enum PostReaction {
    Table,
    PostId,
    UserId,
    ReactionId,
    CreatedAt,
}

#[derive(DeriveIden)]
pub enum Reaction {
    Table,
    Id,
    Name,
    ImageId,
    RegisteredAt,
}
