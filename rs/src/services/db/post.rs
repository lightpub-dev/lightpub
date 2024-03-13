use std::collections::HashSet;

use async_recursion::async_recursion;
use async_trait::async_trait;
use derive_more::Constructor;
use sqlx::MySqlPool;
use tracing::warn;
use uuid::{fmt::Simple, Uuid};

use crate::{
    holder,
    models::{ApubRenderablePost, HasRemoteUri, PostPrivacy},
    services::{
        apub::{
            post::PostContentService,
            render::{ApubRendererService, TargetedUser},
        },
        id::IDGetterService,
        AllUserFinderService, ApubRequestService, PostCreateError, PostCreateRequest,
        PostCreateService, ServiceError, SignerService,
    },
    utils::{
        generate_uuid,
        post::{find_hashtags, find_mentions, PostSpecifier},
        user::UserSpecifier,
    },
};

#[derive(Constructor)]
pub struct DBPostCreateService {
    pool: MySqlPool,
    finder: holder!(AllUserFinderService),
    renderer: ApubRendererService,
    req: holder!(ApubRequestService),
    signer: holder!(SignerService),
    id_getter: IDGetterService,
    post_content: PostContentService,
}

impl DBPostCreateService {
    #[async_recursion]
    async fn fetch_post_id(
        &mut self,
        spec: &PostSpecifier,
        not_found_err: PostCreateError,
    ) -> Result<Simple, ServiceError<PostCreateError>> {
        match spec {
            PostSpecifier::ID(id) => sqlx::query!(
                "SELECT id AS `id: Simple` FROM posts WHERE id=?",
                id.simple().to_string()
            )
            .fetch_optional(&self.pool)
            .await?
            .map(|p| p.id)
            .ok_or(ServiceError::from_se(not_found_err)),
            PostSpecifier::URI(uri) => {
                let local_post_id = self.id_getter.extract_local_post_id(uri);
                if let Some(local_post_id) = local_post_id {
                    let local_post_id = Uuid::parse_str(&local_post_id)
                        .map_err(|_e| ServiceError::from_se(not_found_err.clone()))?;
                    return self
                        .fetch_post_id(&PostSpecifier::from_id(local_post_id), not_found_err)
                        .await;
                }
                sqlx::query!("SELECT id AS `id: Simple` FROM posts WHERE uri=?", uri)
                    .fetch_optional(&self.pool)
                    .await?
                    .map(|p| p.id)
                    .ok_or(ServiceError::from_se(not_found_err))
            }
        }
    }
}

#[async_trait]
impl PostCreateService for DBPostCreateService {
    async fn create_post(
        &mut self,
        req: &crate::services::PostCreateRequest,
    ) -> Result<Simple, crate::services::ServiceError<crate::services::PostCreateError>> {
        let mut tx = self.pool.begin().await?;
        let uri = req.uri();
        if let Some(uri) = uri {
            if sqlx::query!(
                "SELECT EXISTS(SELECT 1 FROM posts WHERE uri=?) AS `exists: bool`",
                uri
            )
            .fetch_one(&mut *tx)
            .await?
            .exists
            {
                // post already exists
                return Err(ServiceError::from_se(PostCreateError::AlreadyExists));
            }
        }

        use PostCreateRequest::*;
        let (repost_of_spec, reply_to_spec, content) = match req {
            Normal(r) => (None, None, r.content.clone().into()),
            Repost(r) => (Some(&r.repost_of), None, None),
            Quote(r) => (Some(&r.repost_of), None, r.content.clone().into()),
            Reply(r) => (None, Some(&r.reply_to), r.content.clone().into()),
        };
        let poster = req.poster();

        let poster = self
            .finder
            .find_user_by_specifier(&poster)
            .await
            .map_err(|e| match e {
                ServiceError::SpecificError(_) => {
                    ServiceError::SpecificError(PostCreateError::PosterNotFound)
                }
                ServiceError::MiscError(e) => e.into(),
            })?;

        let content = content.map(|s| self.post_content.html_to_internal(&s));

        let repost_of_id = match repost_of_spec {
            None => None,
            Some(s) => self
                .fetch_post_id(&s, PostCreateError::RepostOfNotFound)
                .await?
                .into(),
        };

        let reply_to_id = match reply_to_spec {
            None => None,
            Some(s) => self
                .fetch_post_id(&s, PostCreateError::ReplyToNotFound)
                .await?
                .into(),
        };

        let hashtags: Vec<_> = {
            let from_content = content
                .as_ref()
                .map(|c| find_hashtags(c))
                .unwrap_or_else(|| vec![]);
            let from_hint = req.hint().hashtags();

            let mut set = HashSet::new();
            for tag in from_content.iter().chain(from_hint.iter()) {
                set.insert(tag.clone());
            }
            set.into_iter().collect()
        };
        let mentions: Vec<_> = {
            let mut from_content = content
                .as_ref()
                .map(|c| find_mentions(c))
                .unwrap_or_else(|| vec![]);
            let from_hint = req.hint().mentions();

            // mentions may have duplicates
            // UserSpecifier cannot be used for equality check
            from_content.extend_from_slice(from_hint);
            from_content
        };

        let post_id = generate_uuid();
        let post_id_str = post_id.to_string();
        let poster_id = poster.id;
        let privacy = req.privacy().to_db();
        let created_at = req
            .created_at()
            .map(|t| t.naive_utc())
            .unwrap_or_else(|| chrono::Utc::now().naive_utc());

        sqlx::query!(
                "INSERT INTO posts (id, uri, poster_id, content, privacy, created_at, repost_of_id, reply_to_id) VALUES(?, ?, ?, ?, ?, ?, ?, ?)",
                post_id_str,
                uri,
                poster_id,
                content,
                privacy,
                created_at,
                repost_of_id.map(|s|s.to_string()),
                reply_to_id.map(|s|s.to_string())
            )
            .execute(&mut *tx)
            .await?;

        // FIXME: batch insert
        for hashtag in &hashtags {
            sqlx::query!(
                "INSERT INTO post_hashtags (post_id, hashtag_name) VALUES (?, ?)",
                post_id_str,
                hashtag
            )
            .execute(&mut *tx)
            .await?;
        }

        // FIXME: batch insert
        let mut added_mentions = HashSet::new();
        for mention in &mentions {
            let target_user_id = self.finder.find_user_by_specifier(mention).await;
            let target_user_id = match target_user_id {
                Ok(user) => user.id,
                Err(_) => {
                    warn!("failed to find user {:?}", mention);
                    continue;
                }
            };

            // remove duplicates
            if added_mentions.contains(&target_user_id) {
                continue;
            }
            added_mentions.insert(target_user_id);

            sqlx::query!(
                "INSERT INTO post_mentions (post_id, target_user_id) VALUES (?, ?)",
                post_id_str,
                target_user_id,
            )
            .execute(&mut *tx)
            .await?;
        }

        tx.commit().await?;

        // if this is a remote post, finish
        if uri.is_some() {
            return Ok(post_id);
        }

        let post = LocalPost {
            id: post_id,
            poster: LocalPoster { id: poster_id },
            content: content.expect("content is null"),
            privacy: req.privacy(),
            created_at: chrono::DateTime::<chrono::Utc>::from_naive_utc_and_offset(
                created_at,
                chrono::Utc,
            ),
        };
        let note = self
            .renderer
            .render_create_post(&post)
            .expect("failed to render");
        let inboxes = self
            .find_target_inboxes(note.targeted_users())
            .await
            .unwrap();

        for inbox in inboxes {
            let signer = self
                .signer
                .fetch_signer(&UserSpecifier::from_id(poster.id))
                .await
                .expect("failed to fetch signer"); // FIXME: this is very bad
            let result = self
                .req
                .post_to_inbox(&inbox, &note.note_create().clone().into(), signer)
                .await;
            if let Err(e) = result {
                warn!("failed to post to inbox: {:?}", e)
            }
        }

        Ok(post_id)
    }
}

impl DBPostCreateService {
    async fn find_target_inboxes(
        &mut self,
        targets: &Vec<TargetedUser>,
    ) -> Result<Vec<String>, ()> {
        let mut inboxes = vec![];
        let mut inbox_set = HashSet::new();

        let mut add_inbox = |inbox: &Option<String>, shared_inbox: &Option<String>| {
            let added_inbox = if let Some(shared_inbox) = shared_inbox {
                shared_inbox
            } else if let Some(inbox) = inbox {
                inbox
            } else {
                warn!("no inbox or shared inbox found");
                return;
            };
            if inbox_set.contains(added_inbox) {
                return;
            }
            inboxes.push(added_inbox.clone());
            inbox_set.insert(added_inbox.clone());
        };

        for target in targets {
            match target {
                TargetedUser::Mentioned(user) => {
                    let user = self.finder.find_user_by_specifier(user).await;
                    if let Ok(user) = user {
                        add_inbox(&user.inbox, &user.shared_inbox);
                    } else {
                        warn!("failed to find user {:?}", user);
                    }
                }
                TargetedUser::FollowerOf(user) => {
                    let followers_inboxes = self.finder.find_followers_inboxes(user).await;
                    if let Ok(followers_inboxes) = followers_inboxes {
                        for inbox in followers_inboxes {
                            add_inbox(&inbox.inbox, &inbox.shared_inbox);
                        }
                    } else {
                        warn!("failed to find followers inboxes for {:?}", user);
                    }
                }
            }
        }

        Ok(inboxes)
    }
}

#[derive(Debug, Clone)]
struct LocalPost {
    id: Simple,
    content: String,
    poster: LocalPoster,
    privacy: PostPrivacy,
    created_at: chrono::DateTime<chrono::Utc>,
}

impl HasRemoteUri for LocalPost {
    fn get_local_id(&self) -> String {
        self.id.to_string()
    }

    fn get_remote_uri(&self) -> Option<String> {
        None
    }
}

impl ApubRenderablePost for LocalPost {
    type Poster = LocalPoster;

    fn id(&self) -> Simple {
        self.id
    }

    fn uri(&self) -> Option<String> {
        None
    }

    fn content(&self) -> Option<String> {
        self.content.clone().into()
    }

    fn poster(&self) -> Self::Poster {
        self.poster.clone()
    }

    fn privacy(&self) -> PostPrivacy {
        self.privacy
    }

    fn created_at(&self) -> chrono::DateTime<chrono::Utc> {
        self.created_at
    }
}

#[derive(Debug, Clone)]
struct LocalPoster {
    id: Simple,
}

impl HasRemoteUri for LocalPoster {
    fn get_local_id(&self) -> String {
        self.id.to_string()
    }

    fn get_remote_uri(&self) -> Option<String> {
        None
    }
}
