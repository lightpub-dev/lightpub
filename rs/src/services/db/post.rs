use std::collections::HashSet;

use derive_more::Constructor;
use sqlx::MySqlPool;
use tracing::warn;
use uuid::fmt::Simple;

use crate::{
    models::{ApubRenderablePost, HasRemoteUri, PostPrivacy},
    services::{
        apub::render::{ApubRendererService, TargetedUser},
        AllUserFinderService, ApubRequestService, PostCreateError, PostCreateRequest,
        PostCreateService, ServiceError, SignerService,
    },
    utils::{generate_uuid, user::UserSpecifier},
};

#[derive(Debug, Constructor)]
pub struct DBPostCreateService<T, R, S> {
    pool: MySqlPool,
    finder: T,
    renderer: ApubRendererService,
    req: R,
    signer: S,
}

impl<T: AllUserFinderService, R: ApubRequestService, S: SignerService> PostCreateService
    for DBPostCreateService<T, R, S>
{
    async fn create_post(
        &mut self,
        req: &crate::services::PostCreateRequest,
    ) -> Result<Simple, crate::services::ServiceError<crate::services::PostCreateError>> {
        use PostCreateRequest::*;
        let (repost_of_id, reply_to_id, content) = match req {
            Normal(r) => (None, None, r.content.clone().into()),
            Repost(r) => (r.repost_of.into(), None, None),
            Quote(r) => (r.repost_of.into(), None, r.content.clone().into()),
            Reply(r) => (None, r.reply_to.into(), r.content.clone().into()),
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

        if let Some(repost_of_id) = repost_of_id {
            // check if the post exists
            let repost_target =
                sqlx::query!("SELECT id FROM posts WHERE id=?", repost_of_id.to_string())
                    .fetch_optional(&self.pool)
                    .await?;
            if repost_target.is_none() {
                return Err(ServiceError::from_se(PostCreateError::RepostOfNotFound));
            }
        }

        if let Some(reply_to_id) = reply_to_id {
            // check if the post exists
            let reply_target =
                sqlx::query!("SELECT id FROM posts WHERE id=?", reply_to_id.to_string())
                    .fetch_optional(&self.pool)
                    .await?;
            if reply_target.is_none() {
                return Err(ServiceError::from_se(PostCreateError::ReplyToNotFound));
            }
        }

        let post_id = generate_uuid();
        let post_id_str = post_id.to_string();
        let poster_id = poster.id;
        let privacy = req.privacy().to_db();
        let created_at = chrono::Utc::now().naive_utc();

        sqlx::query!(
            "INSERT INTO posts (id, poster_id, content, privacy, created_at, repost_of_id, reply_to_id) VALUES(?, ?, ?, ?, ?, ?, ?)",
            post_id_str,
            poster_id,
            content,
            privacy,
            created_at,
            repost_of_id.map(|s|s.to_string()),
            reply_to_id.map(|s|s.to_string())
        )
        .execute(&self.pool)
        .await?;

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
        let signer = self
            .signer
            .fetch_signer(&UserSpecifier::from_id(poster.id))
            .await
            .expect("failed to fetch signer");
        for inbox in inboxes {
            let result = self
                .req
                .post_to_inbox(
                    inbox.parse::<reqwest::Url>().unwrap(),
                    note.note_create(),
                    &signer,
                )
                .await;
            if let Err(e) = result {
                warn!("failed to post to inbox: {:?}", e)
            }
        }

        Ok(post_id)
    }
}

impl<T: AllUserFinderService, R, S> DBPostCreateService<T, R, S> {
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
