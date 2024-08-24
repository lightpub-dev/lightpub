use derive_more::Constructor;

use crate::{
    domain::{
        model::{
            post::{
                Post, PostCommon, PostContent, PostId, PostNonRepost, PostPrivacy, PostQuote,
                PostQuoteInfo, PostReaction, PostReplyInfo, PostRepost, PostRepostInfo,
            },
            user::UserId,
            DateTime, URI,
        },
        service::id::IDGenerationService,
    },
    holder,
};

pub trait PostFactory {
    fn create(&mut self, params: &PostCreationParams) -> Post;
}

pub struct PostCreationParams {
    pub id: Option<PostId>,           // if None, generate a new one
    pub uri: Option<URI>,             // if remote post, required
    pub content: Option<PostContent>, // normal or quote => required
    pub author: UserId,
    pub privacy: PostPrivacy,
    pub created_at: DateTime,

    pub reply_to: Option<PostId>,
    pub repost_of: Option<PostId>,
    pub quote_of: Option<PostId>,

    pub mentioned_users: Vec<UserId>,

    pub reply_count: i64,
    pub repost_count: i64,
    pub quote_count: i64,
    pub reactions: Vec<PostReaction>,
}

#[derive(Constructor)]
pub struct DefaultPostFactory {
    id_generator: holder!(IDGenerationService),
}

impl PostFactory for DefaultPostFactory {
    fn create(&mut self, params: &PostCreationParams) -> Post {
        let common = PostCommon::new(
            params
                .id
                .unwrap_or_else(|| self.id_generator.generate_post_id()),
            params.uri.clone(),
            params.author,
            params.privacy,
            params.created_at.clone(),
            None,
        );

        match (params.reply_to, params.repost_of, params.quote_of) {
            (Some(reply_to), None, None) => Post::Normal(PostNonRepost::new(
                common,
                params.content.clone().unwrap(),
                Some(PostReplyInfo::new(reply_to)),
                params.mentioned_users.clone(),
                params.reply_count,
                params.repost_count,
                params.quote_count,
                params.reactions.clone(),
            )),
            (None, Some(repost_of), None) => {
                Post::Repost(PostRepost::new(common, PostRepostInfo::new(repost_of)))
            }
            (None, None, Some(quote_of)) => Post::Quote(PostQuote::new(
                common,
                params.content.clone().unwrap(),
                PostQuoteInfo::new(quote_of),
                params.mentioned_users.clone(),
                params.reply_count,
                params.repost_count,
                params.quote_count,
                params.reactions.clone(),
            )),
            _ => {
                panic!("programming error: invalid post creation params");
            }
        }
    }
}
