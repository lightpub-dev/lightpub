use async_trait::async_trait;

use crate::{
    domain::{
        factory::post::PostCreationParams,
        model::{
            post::{Post, PostContent, PostId, PostPrivacy},
            user::UserId,
            DateTime, URI,
        },
    },
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
        ).execute(&mut self.conn).await.unwrap();

        Ok(post.id())
    }

    async fn delete(&mut self, post: &Post) -> Result<(), RepositoryError> {
        let id = post.id().to_db();
        sqlx::query!(r#"DELETE FROM posts WHERE id=?"#, id)
            .execute(&mut self.conn)
            .await
            .unwrap();

        Ok(())
    }

    async fn find_by_id(&mut self, post_id: &PostId) -> Result<Option<Post>, RepositoryError> {
        let id = post_id.to_db();
        let result = sqlx::query!(r#"SELECT id,poster_id AS `poster_id!`,content,created_at AS `created_at: DateTime`,privacy,reply_to_id,repost_of_id,uri FROM posts WHERE id=?"#, id)
            .fetch_optional(&mut self.conn)
            .await
            .unwrap();

        let post = match result {
            None => return Ok(None),
            Some(p) => p,
        };

        if let Some(reply_to_id) = post.reply_to_id {
            // is a reply

            // check that it is not a repost
            if post.repost_of_id.is_some() {
                panic!("reply is a repost");
            }

            return Ok(Some(self.post_factory.create(&PostCreationParams {
                id: Some(PostId::from_str(&post.id).unwrap()),
                uri: post.uri.map(|u| URI::from_str(u).unwrap()),
                content: Some(PostContent::from_string(post.content.unwrap())),
                author: UserId::from_str(&post.poster_id).unwrap(),
                privacy: post_privacy_from_db(&post.privacy),
                created_at: post.created_at,
                reply_to: Some(PostId::from_str(&reply_to_id).unwrap()),
                repost_of: None,
                quote_of: None,
                mentioned_users: vec![],
                reply_count: 0,
                repost_count: 0,
                quote_count: 0,
                reactions: vec![],
            })));
        }

        if let Some(repost_of_id) = post.repost_of_id {
            // is a repost

            // check that it is not a reply
            if post.reply_to_id.is_some() {
                panic!("repost is a reply");
            }

            let repost_of = match post.content {
                None => Some(PostId::from_str(&repost_of_id).unwrap()),
                Some(_) => None,
            };
            let quote_of = match post.content {
                None => None,
                Some(_) => Some(PostId::from_str(&repost_of_id).unwrap()),
            };

            return Ok(Some(self.post_factory.create(&PostCreationParams {
                id: Some(PostId::from_str(&post.id).unwrap()),
                uri: post.uri.map(|u| URI::from_str(u).unwrap()),
                content: post.content.map(|c| PostContent::from_string(c)),
                author: UserId::from_str(&post.poster_id).unwrap(),
                privacy: post_privacy_from_db(&post.privacy),
                created_at: post.created_at,
                reply_to: None,
                repost_of,
                quote_of,
                mentioned_users: vec![],
                reply_count: 0,
                repost_count: 0,
                quote_count: 0,
                reactions: vec![],
            })));
        }

        return Ok(Some(self.post_factory.create(&PostCreationParams {
            id: Some(PostId::from_str(&post.id).unwrap()),
            uri: post.uri.map(|u| URI::from_str(u).unwrap()),
            content: Some(PostContent::from_string(post.content.unwrap())),
            author: UserId::from_str(&post.poster_id).unwrap(),
            privacy: post_privacy_from_db(&post.privacy),
            created_at: post.created_at,
            reply_to: None,
            repost_of: None,
            quote_of: None,
            mentioned_users: vec![],
            reply_count: 0,
            repost_count: 0,
            quote_count: 0,
            reactions: vec![],
        })));
    }
}
