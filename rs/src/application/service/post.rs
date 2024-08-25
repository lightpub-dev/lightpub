use derive_more::Constructor;
use dto::PostIdData;

use crate::{
    domain::{
        factory::post::{PostCreationParams, PostFactory},
        model::{
            post::{Post, PostContent, PostId, PostPrivacy},
            user::UserId,
            DateTime,
        },
    },
    holder,
    repository::interface::uow::UnitOfWork,
};

#[derive(Constructor)]
pub struct PostCreateApplicationService {
    uow: holder!(UnitOfWork),
    post_factory: holder!(PostFactory),
}

impl PostCreateApplicationService {
    pub async fn create_post(
        &mut self,
        create_command: &NormalPostCreateCommand<'_>,
    ) -> Result<PostIdData, anyhow::Error> {
        let post = self.post_factory.create(&PostCreationParams {
            id: None,
            uri: None,
            content: Some(PostContent::from_string(create_command.content)),
            author: UserId::from_str(create_command.author_id).unwrap(),
            privacy: create_command.privacy,
            created_at: DateTime::from_utc(create_command.created_at),
            reply_to: None,
            repost_of: None,
            quote_of: None,
            mentioned_users: vec![],
            reply_count: 0,
            repost_count: 0,
            quote_count: 0,
            reactions: vec![],
        });
        self.store_post(&post).await
    }

    pub async fn create_reply(
        &mut self,
        create_command: &ReplyPostCreateCommand<'_>,
    ) -> Result<PostIdData, anyhow::Error> {
        let post = self.post_factory.create(&PostCreationParams {
            id: None,
            uri: None,
            content: Some(PostContent::from_string(create_command.content)),
            author: UserId::from_str(create_command.author_id).unwrap(),
            privacy: create_command.privacy,
            created_at: DateTime::from_utc(create_command.created_at),
            reply_to: Some(PostId::from_str(create_command.reply_to).unwrap()),
            repost_of: None,
            quote_of: None,
            mentioned_users: vec![],
            reply_count: 0,
            repost_count: 0,
            quote_count: 0,
            reactions: vec![],
        });
        self.store_post(&post).await
    }

    pub async fn create_repost(
        &mut self,
        create_command: &RepostCreateCommand<'_>,
    ) -> Result<PostIdData, anyhow::Error> {
        let post = self.post_factory.create(&PostCreationParams {
            id: None,
            uri: None,
            content: None,
            author: UserId::from_str(create_command.author_id).unwrap(),
            privacy: create_command.privacy,
            created_at: DateTime::from_utc(create_command.created_at),
            reply_to: None,
            repost_of: Some(PostId::from_str(create_command.repost_of).unwrap()),
            quote_of: None,
            mentioned_users: vec![],
            reply_count: 0,
            repost_count: 0,
            quote_count: 0,
            reactions: vec![],
        });
        self.store_post(&post).await
    }

    pub async fn create_quote(
        &mut self,
        create_command: &QuoteCreateCommand<'_>,
    ) -> Result<PostIdData, anyhow::Error> {
        let post = self.post_factory.create(&PostCreationParams {
            id: None,
            uri: None,
            content: Some(PostContent::from_string(create_command.content)),
            author: UserId::from_str(create_command.author_id).unwrap(),
            privacy: create_command.privacy,
            created_at: DateTime::from_utc(create_command.created_at),
            reply_to: None,
            repost_of: None,
            quote_of: Some(PostId::from_str(create_command.quote_of).unwrap()),
            mentioned_users: vec![],
            reply_count: 0,
            repost_count: 0,
            quote_count: 0,
            reactions: vec![],
        });
        self.store_post(&post).await
    }

    async fn store_post(&mut self, post: &Post) -> Result<PostIdData, anyhow::Error> {
        let mut post_repository = self.uow.repository_manager().post_repository();

        post_repository
            .create(post)
            .await
            .map(PostIdData::from_post_id)
            .map_err(|e| e.into())
    }
}

pub struct NormalPostCreateCommand<'a> {
    pub author_id: &'a str,
    pub privacy: PostPrivacy,
    pub created_at: chrono::DateTime<chrono::Utc>,
    pub content: &'a str,
}

pub struct ReplyPostCreateCommand<'a> {
    pub author_id: &'a str,
    pub privacy: PostPrivacy,
    pub created_at: chrono::DateTime<chrono::Utc>,
    pub content: &'a str,
    pub reply_to: &'a str,
}

pub struct RepostCreateCommand<'a> {
    pub author_id: &'a str,
    pub privacy: PostPrivacy,
    pub created_at: chrono::DateTime<chrono::Utc>,
    pub repost_of: &'a str,
}

pub struct QuoteCreateCommand<'a> {
    pub author_id: &'a str,
    pub privacy: PostPrivacy,
    pub created_at: chrono::DateTime<chrono::Utc>,
    pub content: &'a str,
    pub quote_of: &'a str,
}

pub mod dto {
    use crate::domain::model::post::PostId;

    #[derive(Debug, Clone, PartialEq, Eq, Copy)]
    pub struct PostIdData {
        pub id: PostId,
    }

    impl PostIdData {
        pub fn from_post_id(id: PostId) -> Self {
            Self { id }
        }

        pub fn id(&self) -> PostId {
            self.id
        }
    }
}
