use async_trait::async_trait;

use crate::{
    domain::model::post::{Post, PostId, PostPrivacy},
    repository::interface::{post::PostRepository, RepositoryError},
};

use super::{IsUuid, SqliteRepository};

impl IsUuid for PostId {
    fn from_uuid(uuid: uuid::Uuid) -> Self {
        Self::from_uuid(uuid)
    }

    fn to_uuid(&self) -> uuid::Uuid {
        self.id()
    }
}

pub(super) fn post_privacy_to_db(privacy: PostPrivacy) -> &'static str {
    use PostPrivacy::*;
    match privacy {
        Public => "public",
        Unlisted => "unlisted",
        Followers => "follower",
        Private => "private",
    }
}

pub(super) fn post_privacy_from_db(privacy: &str) -> PostPrivacy {
    match privacy {
        "public" => PostPrivacy::Public,
        "unlisted" => PostPrivacy::Unlisted,
        "follower" => PostPrivacy::Followers,
        "private" => PostPrivacy::Private,
        _ => unreachable!(),
    }
}

#[async_trait]
impl<'a> PostRepository for SqliteRepository<'a> {
    async fn create(&mut self, post: &Post) -> Result<PostId, RepositoryError> {
        let id = post.id().to_db();
        let poster_id = post.author_id().to_db();
        let content = post.content().map(|c| c.to_str());
        let created_at = post.created_at();
        let privacy = post_privacy_to_db(post.privacy());
        let reply_to = post.reply_to().map(|r| r.id().to_db());
        let repost_of = post.repost_of().map(|r| r.id().to_db());
        let uri = post.uri().map(|u| u.to_str());

        sqlx::query!(r#"INSERT INTO posts(id,poster_id,content,inserted_at,created_at,privacy,reply_to_id,repost_of_id,uri) VALUES (?,?,?,?,?,?,?,?,?)"#,
            id,
            poster_id,
            content,
            created_at,
            created_at,
            privacy,
            reply_to,
            repost_of,
            uri
        ).execute(self).await.unwrap();

        Ok(post.id())
    }

    async fn 
}
