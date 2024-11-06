use std::collections::HashSet;

use async_recursion::async_recursion;
use async_trait::async_trait;
use derive_more::Constructor;
use gen_span::gen_span;
use sqlx::SqlitePool;
use tracing::warn;
use uuid::{fmt::Simple, Uuid};

use crate::backend::{
    apub::{
        post::PostContentService,
        render::{ApubRendererService, TargetedUser},
    },
    id::IDGetterService,
    AllUserFinderService, ApubRequestService, FetchUserPostsOptions, PostCreateError,
    PostCreateRequest, PostCreateService, PostDeleteError, PostFetchError, PostInteractionAction,
    ServiceError, SignerService, TimelineOptions, UserFollowService, UserPostService,
};
use crate::holder;
use crate::model::{
    api_response::{
        PostAuthorBuilder, PostCountsBuilder, PostMentionedUser, PostReaction, UserPostEntry,
        UserPostEntryBuilder,
    },
    apub::{Activity, CreatableObject},
    reaction::Reaction,
    ApubMentionedUser, ApubPostTargetComputable, ApubRenderablePost, HasRemoteUri,
    HasRemoteUriOnly, PostPrivacy, PostSpecifier, UserSpecifier,
};
use crate::utils::{
    generate_uuid,
    post::{find_hashtags, find_mentions},
};

#[derive(Constructor)]
pub struct DBPostCreateService {
    pool: SqlitePool,
    finder: holder!(AllUserFinderService),
    renderer: ApubRendererService,
    req: holder!(ApubRequestService),
    signer: holder!(SignerService),
    id_getter: IDGetterService,
    post_content: PostContentService,
    follow: holder!(UserFollowService),
}

#[derive(Debug)]
struct SimplePost {
    id: Simple,
    uri: Option<String>,
    poster: Simple,
    privacy: PostPrivacy,
    repost_of_id: Option<Simple>,
}

impl HasRemoteUri for SimplePost {
    fn get_local_id(&self) -> String {
        self.id.to_string()
    }

    fn get_remote_uri(&self) -> Option<String> {
        self.uri.clone()
    }
}

#[gen_span]
impl DBPostCreateService {
    async fn visibility_check(
        &mut self,
        post_spec: &PostSpecifier,
        viewer: &UserSpecifier,
        repost_mode: bool,
    ) -> Result<bool, ServiceError<PostCreateError>> {
        let post = self.fetch_post_locally_stored(post_spec, false).await?;
        let post = if let Some(post) = post {
            post
        } else {
            return Err(ServiceError::from_se(PostCreateError::PostNotFound));
        };

        let post_id = &post.id;
        let poster = post.poster.into();
        let privacy = post.privacy;

        let poster_id = self
            .finder
            .find_user_by_specifier(&poster)
            .await
            .map_err(|_e| ServiceError::from_se(PostCreateError::PosterNotFound))?
            .id;
        let viewer_id = self
            .finder
            .find_user_by_specifier(viewer)
            .await
            .map_err(|_e| ServiceError::from_se(PostCreateError::PosterNotFound))?
            .id;

        if poster_id == viewer_id {
            return Ok(match privacy {
                PostPrivacy::Public | PostPrivacy::Unlisted => true,
                _ => !repost_mode,
            });
        }

        match privacy {
            PostPrivacy::Public | PostPrivacy::Unlisted => Ok(true),
            PostPrivacy::Followers => {
                if repost_mode {
                    return Ok(false);
                }
                // check if viewer is a follower of poster
                self.follow
                    .is_following(&poster, viewer)
                    .await
                    .map_err(|_e| ServiceError::from_se(PostCreateError::PosterNotFound))
            }
            PostPrivacy::Private => {
                if repost_mode {
                    return Ok(false);
                }
                // check if viewer is mentioned in the post

                let mentiond = {
                    let post_id_str = post_id.to_string();
                    let viewer_id_str = viewer_id.to_string();
                    sqlx::query!(
                        r#"SELECT id FROM post_mentions WHERE post_id=? AND target_user_id=?
                    "#,
                        post_id_str,
                        viewer_id_str
                    )
                    .fetch_optional(&self.pool)
                    .await?
                };

                Ok(mentiond.is_some())
            }
        }
    }

    #[async_recursion]
    async fn fetch_post_id(
        &mut self,
        spec: &PostSpecifier,
        not_found_err: PostCreateError,
        depth: i32,
        include_deleted: bool,
    ) -> Result<Simple, ServiceError<PostCreateError>> {
        match spec {
            PostSpecifier::ID(id) => {
                let id_str = id.simple().to_string();
                sqlx::query!(
                    "SELECT id AS `id: Simple` FROM posts WHERE id=? AND (? OR deleted_at IS NULL)",
                    id_str,
                    include_deleted,
                )
                .fetch_optional(&self.pool)
                .await?
                .map(|p| p.id)
                .ok_or(ServiceError::from_se(not_found_err))
            }
            PostSpecifier::URI(uri) => {
                let local_post_id = self.id_getter.extract_local_post_id(uri);
                if let Some(local_post_id) = local_post_id {
                    let local_post_id = Uuid::parse_str(&local_post_id)
                        .map_err(|_e| ServiceError::from_se(not_found_err.clone()))?;
                    return self
                        .fetch_post_id(
                            &PostSpecifier::from_id(local_post_id),
                            not_found_err,
                            depth + 1,
                            include_deleted,
                        )
                        .await;
                }

                let result = sqlx::query!("SELECT id AS `id: Simple` FROM posts WHERE uri=? AND (? OR deleted_at IS NULL)", uri, include_deleted)
                    .fetch_optional(&self.pool)
                    .await?
                    .map(|p| p.id);
                if let Some(result) = result {
                    return Ok(result);
                }

                if depth > 10 {
                    // TODO: 10 でいいの?
                    return Err(ServiceError::from_se(not_found_err));
                }
                let remote_post = self.req.fetch_post(uri).await.map_err(|e| {
                    warn!("failed to fetch remote post: {:?}", e);
                    ServiceError::from_se(not_found_err.clone())
                })?;
                match remote_post {
                    CreatableObject::Note(remote_post) => {
                        self.create_post_(&remote_post.try_into().unwrap(), depth + 1)
                            .await
                    }
                    _ => {
                        warn!("remote post is not a note");
                        Err(ServiceError::from_se(not_found_err))
                    }
                }
            }
        }
    }

    // #[async_recursion]
    // async fn fetch_post_id_locally_stored(
    //     &mut self,
    //     spec: &PostSpecifier,
    //     include_deleted: bool,
    // ) -> Result<Option<Simple>, ServiceError<PostCreateError>> {
    //     self.fetch_post_locally_stored(spec, include_deleted)
    //         .await
    //         .map(|p| p.map(|p| p.id))
    // }

    #[async_recursion]
    async fn fetch_post_locally_stored(
        &mut self,
        spec: &PostSpecifier,
        include_deleted: bool,
    ) -> Result<Option<SimplePost>, ServiceError<PostCreateError>> {
        let id = match spec {
            PostSpecifier::ID(id) => {
                let id_str = id.simple().to_string();
                sqlx::query!(
                "SELECT id AS `id: Simple`, uri AS `uri`, poster_id AS `poster!: Simple`, privacy, repost_of_id AS `repost_of_id: Simple` FROM posts WHERE id=? AND (? OR deleted_at IS NULL) AND poster_id IS NOT NULL",
                id_str,
                include_deleted,
            )
            .fetch_optional(&self.pool)
            .await?
            .map(|p| SimplePost {
                id: p.id,
                uri: p.uri,
                poster: p.poster,
                privacy: p.privacy.parse().unwrap(),
                repost_of_id: p.repost_of_id,
            })
            }
            PostSpecifier::URI(uri) => {
                let local_post_id = self.id_getter.extract_local_post_id(uri);
                if let Some(local_post_id) = local_post_id {
                    let local_post_id = Uuid::parse_str(&local_post_id)
                        .map_err(|_e| ServiceError::from_se(PostCreateError::PostNotFound))?;
                    return self
                        .fetch_post_locally_stored(
                            &PostSpecifier::from_id(local_post_id),
                            include_deleted,
                        )
                        .await;
                }

                sqlx::query!(
                    "SELECT id AS `id: Simple`, uri AS `uri`, poster_id AS `poster!: Simple`, privacy, repost_of_id AS `repost_of_id: Simple` FROM posts WHERE uri=? AND (? OR deleted_at IS NULL) AND poster_id IS NOT NULL",
                    uri,
                    include_deleted,
                )
                .fetch_optional(&self.pool)
                .await?
                .map(|p| SimplePost {
                    id: p.id,
                    uri: p.uri,
                    poster: p.poster,
                    privacy: p.privacy.parse().unwrap(),
                    repost_of_id: p.repost_of_id,
                })
            }
        };
        Ok(id)
    }

    async fn create_post_(
        &mut self,
        req: &crate::backend::PostCreateRequest,
        depth: i32,
    ) -> Result<Simple, ServiceError<crate::backend::PostCreateError>> {
        let mut tx = self.pool.begin().await?;
        let uri = req.uri();
        if let Some(uri) = uri {
            if sqlx::query!(
                "SELECT EXISTS(SELECT 1 FROM posts WHERE uri=?) AS `exists!: bool`",
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
            Some(s) => {
                let repost_of = self
                    .fetch_post_id(&s, PostCreateError::RepostOfNotFound, depth + 1, false)
                    .await?;
                let is_repost = content.is_none();
                if !self
                    .visibility_check(&PostSpecifier::from_id(repost_of), req.poster(), is_repost)
                    .await?
                {
                    return Err(ServiceError::from_se(PostCreateError::RepostOfNotFound));
                }

                if is_repost {
                    match req.privacy() {
                        PostPrivacy::Followers | PostPrivacy::Private => {
                            return Err(ServiceError::from_se(
                                PostCreateError::DisallowedPrivacyForRepost,
                            ));
                        }
                        _ => {}
                    }
                }

                if self.is_repost(&PostSpecifier::from_id(repost_of)).await? {
                    return Err(ServiceError::from_se(PostCreateError::NestedRepost));
                }

                Some(repost_of)
            }
        };

        let reply_to_id = match reply_to_spec {
            None => None,
            Some(s) => {
                let reply_to = self
                    .fetch_post_id(&s, PostCreateError::ReplyToNotFound, depth + 1, false)
                    .await?;

                if !self
                    .visibility_check(&PostSpecifier::from_id(reply_to), req.poster(), false)
                    .await?
                {
                    return Err(ServiceError::from_se(PostCreateError::ReplyToNotFound));
                }

                if self.is_repost(&PostSpecifier::from_id(reply_to)).await? {
                    return Err(ServiceError::from_se(PostCreateError::NestedRepost));
                }

                Some(reply_to)
            }
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

        {
            let repost_of_id_str = repost_of_id.as_ref().map(|s| s.to_string());
            let reply_to_id_str = reply_to_id.as_ref().map(|s| s.to_string());
            sqlx::query!(
                "INSERT INTO posts (id, uri, poster_id, content, privacy, created_at, repost_of_id, reply_to_id) VALUES(?, ?, ?, ?, ?, ?, ?, ?)",
                post_id_str,
                uri,
                poster_id,
                content,
                privacy,
                created_at,
                repost_of_id_str,
                reply_to_id_str
            )
            .execute(&mut *tx)
            .await?;
        }

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
        let mut mentioned_users = vec![];
        for mention in &mentions {
            let target_user = self.finder.find_user_by_specifier(mention).await;
            let target_user_id = match &target_user {
                Ok(user) => user.id,
                Err(_) => {
                    warn!("failed to find user {:?}", mention);
                    continue;
                }
            };
            let target_user = target_user.unwrap();

            // remove duplicates
            if added_mentions.contains(&target_user_id) {
                continue;
            }
            added_mentions.insert(target_user_id);
            mentioned_users.push(LocalMentionedUser {
                inbox: target_user.inbox,
                username: target_user.username,
                host: target_user.host,
                uri: target_user.uri,
            });

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

        if let Some(content) = content {
            let repost_of_uri = match repost_of_id {
                None => None,
                Some(id) => {
                    let p = self
                        .fetch_post_locally_stored(&PostSpecifier::ID(id.into()), false)
                        .await?
                        .expect("repost should exist");
                    Some(self.id_getter.get_post_id(&p))
                }
            };
            let reply_to_uri = match reply_to_id {
                None => None,
                Some(id) => {
                    let p = self
                        .fetch_post_locally_stored(&PostSpecifier::ID(id.into()), false)
                        .await?
                        .expect("reply should exist");
                    Some(self.id_getter.get_post_id(&p))
                }
            };

            let post = LocalPost {
                id: post_id,
                poster: LocalPoster { id: poster_id },
                content: Some(content),
                privacy: req.privacy(),
                created_at: chrono::DateTime::<chrono::Utc>::from_naive_utc_and_offset(
                    created_at,
                    chrono::Utc,
                ),
                mentioned_users,
                repost_of_uri,
                reply_to_uri,
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
                    .post_to_inbox(
                        &inbox,
                        &note.note_create().clone().activity().into(),
                        signer,
                    )
                    .await;
                if let Err(e) = result {
                    warn!("failed to post to inbox: {:?}", e)
                }
            }

            Ok(post_id)
        } else {
            let repost_of_uri = match repost_of_id {
                None => unreachable!("repost_of_id must not be None if content is None"),
                Some(id) => {
                    let p = self
                        .fetch_post_locally_stored(&PostSpecifier::ID(id.into()), false)
                        .await?
                        .expect("repost should exist");
                    self.id_getter.get_post_id(&p)
                }
            };
            let reply_to_uri = match reply_to_id {
                None => None,
                Some(id) => {
                    let p = self
                        .fetch_post_locally_stored(&PostSpecifier::ID(id.into()), false)
                        .await?
                        .expect("reply should exist");
                    Some(self.id_getter.get_post_id(&p))
                }
            };

            let post = LocalPost {
                id: post_id,
                poster: LocalPoster { id: poster_id },
                content: None,
                privacy: req.privacy(),
                created_at: chrono::DateTime::<chrono::Utc>::from_naive_utc_and_offset(
                    created_at,
                    chrono::Utc,
                ),
                mentioned_users,
                repost_of_uri: Some(repost_of_uri),
                reply_to_uri,
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
                    .post_to_inbox(
                        &inbox,
                        &note.note_create().clone().activity().into(),
                        signer,
                    )
                    .await;
                if let Err(e) = result {
                    warn!("failed to post to inbox: {:?}", e)
                }
            }

            Ok(post_id)
        }
    }

    async fn is_repost(
        &mut self,
        post_spec: &PostSpecifier,
    ) -> Result<bool, ServiceError<PostCreateError>> {
        let post = self.fetch_post_locally_stored(post_spec, false).await?;
        match post {
            None => Err(ServiceError::from_se(PostCreateError::PostNotFound)),
            Some(post) => Ok(post.repost_of_id.is_some()),
        }
    }
}

#[derive(Debug)]
struct TargetComputablePostDB {
    id: Simple,
    uri: Option<String>,
    poster_id: Simple,
    poster_uri: Option<String>,
    privacy: String,
}

#[derive(Debug, Clone)]
struct TargetComputablePostUser {
    id: Simple,
    uri: Option<String>,
}

impl HasRemoteUri for TargetComputablePostUser {
    fn get_local_id(&self) -> String {
        self.id.to_string()
    }

    fn get_remote_uri(&self) -> Option<String> {
        self.uri.clone()
    }
}

impl HasRemoteUriOnly for TargetComputablePostUser {
    fn get_remote_uri(&self) -> Option<String> {
        self.uri.clone()
    }
}

#[derive(Debug)]
struct TargetComputablePost {
    db: TargetComputablePostDB,
    privacy: PostPrivacy,
    mentioned_users: Vec<TargetComputablePostUser>,
}

impl HasRemoteUri for TargetComputablePost {
    fn get_local_id(&self) -> String {
        self.db.id.to_string()
    }

    fn get_remote_uri(&self) -> Option<String> {
        self.db.uri.clone()
    }
}

impl ApubPostTargetComputable for TargetComputablePost {
    type Poster = TargetComputablePostUser;
    type Actor = TargetComputablePostUser;

    fn privacy(&self) -> PostPrivacy {
        self.privacy
    }

    fn poster(&self) -> Self::Poster {
        TargetComputablePostUser {
            id: self.db.poster_id,
            uri: self.db.poster_uri.clone(),
        }
    }

    fn mentioned(&self) -> Vec<Self::Actor> {
        self.mentioned_users.clone()
    }
}

#[gen_span]
impl DBPostCreateService {
    async fn get_target_computable_post(
        &mut self,
        post_id: Simple,
    ) -> Result<(TargetComputablePost, TargetComputablePostUser), anyhow::Error> {
        let post_id_str = post_id.to_string();
        let detailed_post = sqlx::query_as!(TargetComputablePostDB, r#"
                SELECT p.id AS `id: Simple`, p.uri AS `uri`, p.poster_id AS `poster_id!: Simple`, u.uri AS `poster_uri`, p.privacy
                FROM posts AS p
                INNER JOIN users AS u ON p.poster_id=u.id
                WHERE p.id=?
                "#, post_id_str).fetch_one(&self.pool).await?;
        let mentioned_users = sqlx::query_as!(
            TargetComputablePostUser,
            r#"
                SELECT u.id AS `id: Simple`, u.uri AS `uri`
                FROM post_mentions m
                INNER JOIN users u ON m.target_user_id=u.id
                WHERE m.post_id=?
                "#,
            post_id_str
        )
        .fetch_all(&self.pool)
        .await?;

        let privacy = detailed_post.privacy.parse().unwrap();
        let detailed_post = TargetComputablePost {
            db: detailed_post,
            privacy: privacy,
            mentioned_users,
        };
        let author = TargetComputablePostUser {
            id: detailed_post.db.poster_id,
            uri: detailed_post.db.poster_uri.clone(),
        };

        Ok((detailed_post, author))
    }
}

#[gen_span]
#[async_trait]
impl PostCreateService for DBPostCreateService {
    async fn create_post(
        &mut self,
        req: &crate::backend::PostCreateRequest,
    ) -> Result<Simple, ServiceError<crate::backend::PostCreateError>> {
        self.create_post_(req, 0).await
    }

    async fn delete_post(
        &mut self,
        req: &PostSpecifier,
        actor: &Option<UserSpecifier>,
    ) -> Result<(), anyhow::Error> {
        let post = self.fetch_post_locally_stored(req, false).await?;
        if let Some(post) = post {
            // if actor is specified
            if let Some(actor) = actor {
                // check if actor is the poster
                let actor_id = self.finder.find_user_by_specifier(actor).await?.id;
                if post.poster != actor_id {
                    return Err(PostDeleteError::Unauthorized.into());
                }
            }

            let mut tx = self.pool.begin().await?;

            // delete post and its reposts
            let post_id_str = post.id.to_string();
            sqlx::query!(
                "UPDATE posts SET deleted_at=CURRENT_TIMESTAMP WHERE id=? OR (repost_of_id=? AND content IS NULL AND deleted_at IS NULL)",
                post_id_str,
                post_id_str,
            )
            .execute(&mut *tx)
            .await?;

            tx.commit().await?;

            // send apub if local post
            if post.uri.is_none() {
                let (detailed_post, author) = self.get_target_computable_post(post.id).await?;

                let note_delete = self.renderer.render_delete_post(&detailed_post, &author)?;

                let inboxes = self
                    .find_target_inboxes(&note_delete.targeted_users)
                    .await
                    .unwrap();

                let activity = Activity::Delete(note_delete.note_delete);
                for inbox in inboxes {
                    let signer = self
                        .signer
                        .fetch_signer(&UserSpecifier::from_id(post.poster))
                        .await?; // FIXME: do not generate signer for each inbox
                    self.req.post_to_inbox(&inbox, &activity, signer).await?;
                }
            }

            Ok(())
        } else {
            Err(PostDeleteError::PostNotFound.into())
        }
    }

    async fn modify_favorite(
        &mut self,
        user_spec: &UserSpecifier,
        post_spec: &PostSpecifier,
        allow_remote: bool,
        as_bookmark: bool,
        action: PostInteractionAction,
    ) -> Result<(), anyhow::Error> {
        let user = self.finder.find_user_by_specifier(user_spec).await?;
        let post_id = if allow_remote {
            self.fetch_post_id(post_spec, PostCreateError::PostNotFound, 0, false)
                .await?
        } else {
            self.fetch_post_locally_stored(post_spec, false)
                .await?
                .ok_or(PostCreateError::PostNotFound)
                .map(|p| p.id)?
        };

        let modified_id = match action {
            PostInteractionAction::Add => {
                let id = generate_uuid();
                let id_str = id.to_string();
                let user_id_str = user.id.to_string();
                let post_id_str = post_id.to_string();
                let result = sqlx::query!(
                    r#"
                    INSERT INTO post_favorites (id, user_id, post_id, is_bookmark) VALUES (?,?,?,?) ON CONFLICT DO NOTHING
                    "#,
                    id_str,
                    user_id_str,
                    post_id_str,
                    as_bookmark
                )
                .execute(&self.pool).await?;
                if result.rows_affected() > 0 {
                    Some(id)
                } else {
                    None
                }
            }
            PostInteractionAction::Remove => {
                let mut tx = self.pool.begin().await?;
                // first, select to get id
                let user_id_str = user.id.to_string();
                let post_id_str = post_id.to_string();
                let record = sqlx::query!(
                    r#"
                    SELECT id AS `id: Simple` FROM post_favorites WHERE user_id=? AND post_id=? AND is_bookmark=?
                    "#,
                    user_id_str,
                    post_id_str,
                    as_bookmark,
                ).fetch_optional(&mut *tx).await?;
                if let Some(id) = record {
                    // then, delete
                    let id_str = id.id.to_string();
                    sqlx::query!(
                        r#"
                        DELETE FROM post_favorites WHERE id=?
                        "#,
                        id_str,
                    )
                    .execute(&mut *tx)
                    .await?;
                    tx.commit().await?;
                    Some(id.id)
                } else {
                    tx.rollback().await?;
                    None
                }
            }
        };

        let modified_id = match modified_id {
            None => return Ok(()), // no change occurred
            Some(id) => id,
        };
        if as_bookmark {
            // bookmarks are private
            return Ok(());
        }

        // Apub should be sent when the actor is a local user
        if user.uri.is_none() {
            // local user
            // send apub
            // LikeActivity with no content
            // or UndoActivity with LikeActivity
            let actor = self
                .finder
                .find_user_by_specifier(&UserSpecifier::from_id(user.id))
                .await?;
            let post = self
                .fetch_post_locally_stored(&PostSpecifier::from_id(post_id), false)
                .await?
                .ok_or_else(|| anyhow::anyhow!("post with modified favorite not found"))?;

            let is_add = match action {
                PostInteractionAction::Add => true,
                PostInteractionAction::Remove => false,
            };
            let activity = self.renderer.render_post_reaction(
                &modified_id.to_string(),
                &actor,
                &post,
                None as Option<&str>,
                is_add,
            )?;

            let (post, _) = self.get_target_computable_post(post.id).await?;
            let targets = self.renderer.calculate_post_involved_users(&post, true)?;
            let inboxes = self.find_target_inboxes(&targets).await?;
            for inbox in inboxes {
                let signer = self
                    .signer
                    .fetch_signer(&UserSpecifier::from_id(user.id))
                    .await?;
                self.req.post_to_inbox(&inbox, &activity, signer).await?;
            }
        }

        return Ok(());
    }

    async fn modify_reaction(
        &mut self,
        user_spec: &UserSpecifier,
        post_spec: &PostSpecifier,
        reaction: &Reaction,
        allow_remote: bool,
        action: PostInteractionAction,
    ) -> Result<(), anyhow::Error> {
        let user = self.finder.find_user_by_specifier(user_spec).await?;
        let post_id = if allow_remote {
            self.fetch_post_id(post_spec, PostCreateError::PostNotFound, 0, false)
                .await?
        } else {
            self.fetch_post_locally_stored(post_spec, false)
                .await?
                .ok_or(PostCreateError::PostNotFound)
                .map(|p| p.id)?
        };

        let (reaction_str, custom_reaction_id): (Option<String>, Option<i64>) = match reaction {
            Reaction::Unicode(u) => (Some(u.to_string()), None),
            Reaction::Custom(_c) => todo!("custom reaction support"),
        };

        let modified_id = match action {
            PostInteractionAction::Add => {
                let id = generate_uuid();
                let id_str = id.to_string();
                let user_id_str = user.id.to_string();
                let post_id_str = post_id.to_string();
                let result = sqlx::query!(
                    r#"
                    INSERT INTO post_reactions (id, user_id, post_id, reaction_str, custom_reaction_id) VALUES (?,?,?,?,?) ON CONFLICT DO NOTHING
                    "#,
                    id_str,
                    user_id_str,
                    post_id_str,
                    reaction_str,
                    custom_reaction_id,
                )
                .execute(&self.pool).await?;
                if result.rows_affected() > 0 {
                    Some(id)
                } else {
                    None
                }
            }
            PostInteractionAction::Remove => {
                let mut tx = self.pool.begin().await?;
                // first, select to get id
                let user_id_str = user.id.to_string();
                let post_id_str = post_id.to_string();
                struct QueryResult {
                    id: Simple,
                }
                let record = sqlx::query_as!(
                    QueryResult,
                    r#"
                    SELECT id AS `id: Simple` FROM post_reactions WHERE user_id=? AND post_id=? AND reaction_str=? AND custom_reaction_id=?
                    "#,
                    user_id_str,
                    post_id_str,
                    reaction_str,
                    custom_reaction_id,
                ).fetch_optional(&mut *tx).await?;
                if let Some(record) = record {
                    // then, delete
                    let id_str = record.id.to_string();
                    sqlx::query!(
                        r#"
                        DELETE FROM post_reactions WHERE id=?
                        "#,
                        id_str,
                    )
                    .execute(&mut *tx)
                    .await?;
                    tx.commit().await?;
                    Some(record.id)
                } else {
                    tx.rollback().await?;
                    None
                }
            }
        };

        let modified_id = match modified_id {
            None => return Ok(()), // no change occurred
            Some(id) => id,
        };

        // Apub should be sent when local user and remote post
        if user.uri.is_none() {
            // local user
            // send apub
            // LikeActivity with no content
            // or UndoActivity with LikeActivity
            let actor = self
                .finder
                .find_user_by_specifier(&UserSpecifier::from_id(user.id))
                .await?;
            let post = self
                .fetch_post_locally_stored(&PostSpecifier::from_id(post_id), false)
                .await?
                .ok_or_else(|| anyhow::anyhow!("post with modified favorite not found"))?;

            let is_add = match action {
                PostInteractionAction::Add => true,
                PostInteractionAction::Remove => false,
            };
            let activity = self.renderer.render_post_reaction(
                &modified_id.to_string(),
                &actor,
                &post,
                Some(reaction.to_apub()),
                is_add,
            )?;

            let (post, _) = self.get_target_computable_post(post.id).await?;
            let targets = self.renderer.calculate_post_involved_users(&post, true)?;
            let inboxes = self.find_target_inboxes(&targets).await?;
            for inbox in inboxes {
                let signer = self
                    .signer
                    .fetch_signer(&UserSpecifier::from_id(user.id))
                    .await?;
                self.req.post_to_inbox(&inbox, &activity, signer).await?;
            }
        }

        return Ok(());
    }
}

#[gen_span]
impl DBPostCreateService {
    async fn find_target_inboxes(
        &mut self,
        targets: &Vec<TargetedUser>,
    ) -> Result<Vec<String>, anyhow::Error> {
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
            inbox_set.insert(added_inbox.clone());
        };

        for target in targets {
            match target {
                TargetedUser::Mentioned(user) => {
                    let user = self.finder.find_user_by_specifier(user).await;
                    if let Ok(user) = user {
                        // check if user is remote
                        if user.uri.is_some() {
                            add_inbox(&user.inbox, &user.shared_inbox);
                        }
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

        Ok(inbox_set.into_iter().collect())
    }
}

#[derive(Debug, Clone)]
struct LocalPost {
    id: Simple,
    content: Option<String>,
    poster: LocalPoster,
    privacy: PostPrivacy,
    created_at: chrono::DateTime<chrono::Utc>,
    mentioned_users: Vec<LocalMentionedUser>,
    repost_of_uri: Option<String>,
    reply_to_uri: Option<String>,
}

#[derive(Debug, Clone)]
struct LocalMentionedUser {
    inbox: Option<String>,
    username: String,
    host: Option<String>,
    uri: Option<String>,
}

impl Into<ApubMentionedUser> for LocalMentionedUser {
    fn into(self) -> ApubMentionedUser {
        ApubMentionedUser {
            inbox: self.inbox,
            username: self.username,
            host: self.host,
            uri: self.uri,
        }
    }
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

    fn deleted_at(&self) -> Option<chrono::DateTime<chrono::Utc>> {
        None
    }

    fn repost_of_id(&self) -> Option<String> {
        self.repost_of_uri.clone()
    }

    fn reply_to_id(&self) -> Option<String> {
        self.reply_to_uri.clone()
    }

    fn mentioned(&self) -> Vec<ApubMentionedUser> {
        self.mentioned_users
            .iter()
            .map(|s| s.clone().into())
            .collect()
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

#[derive(Constructor)]
pub struct DBUserPostService {
    pool: SqlitePool,
    finder: holder!(AllUserFinderService),
    id_getter: IDGetterService,
}

#[derive(Debug)]
struct UserPost {
    id: Simple,
    uri: Option<String>,
    author_id: Simple,
    author_uri: Option<String>,
    author_username: String,
    author_host: Option<String>,
    author_nickname: String,
    content: Option<String>,
    privacy: String,
    repost_of_id: Option<Simple>,
    reply_to_id: Option<Simple>,
    created_at: chrono::NaiveDateTime,
    deleted_at: Option<chrono::NaiveDateTime>,
    count_replies: i64,
    count_reposts: i64,
    count_quotes: i64,
    reposted_by_you: Option<bool>,
    favorited_by_you: Option<bool>,
    bookmarked_by_you: Option<bool>,
    reaction_str_by_you: Option<String>,
}

#[derive(Debug)]
struct ReactionCount {
    reaction_str: Option<String>,
    count: i64,
}

struct UserPostAsPost<'a>(pub &'a UserPost);
impl HasRemoteUri for UserPostAsPost<'_> {
    fn get_local_id(&self) -> String {
        self.0.id.to_string()
    }

    fn get_remote_uri(&self) -> Option<String> {
        self.0.uri.clone()
    }
}

struct UserPostAsAuthor<'a>(pub &'a UserPost);
impl HasRemoteUri for UserPostAsAuthor<'_> {
    fn get_local_id(&self) -> String {
        self.0.author_id.to_string()
    }

    fn get_remote_uri(&self) -> Option<String> {
        self.0.author_uri.clone()
    }
}

#[gen_span]
#[async_trait]
impl UserPostService for DBUserPostService {
    async fn fetch_single_post(
        &mut self,
        post_spec: &PostSpecifier,
        viewer: &Option<UserSpecifier>,
    ) -> Result<UserPostEntry, anyhow::Error> {
        let viewer_u = if let Some(viewer) = viewer {
            Some(self.finder.find_user_by_specifier(viewer).await?)
        } else {
            None
        };
        let viewer_id = viewer_u.as_ref().map(|u| u.id.to_string());

        let post_id = match post_spec {
            PostSpecifier::ID(id) => id.simple().to_string(),
            PostSpecifier::URI(_) => todo!("fetch_single_post: URI"),
        };

        let viewer_id_is_valid = viewer_id.is_some();
        let post = sqlx::query_as!(
            UserPost,
            r#"
            SELECT
                p.id AS `id: Simple`,
                p.uri AS `uri`,
                u.id `author_id: Simple`,
                u.uri AS `author_uri`,
                u.username AS `author_username`,
                u.host AS `author_host`,
                u.nickname AS `author_nickname`,
                p.content,
                p.privacy,
                p.repost_of_id AS `repost_of_id: Simple`,
                p.reply_to_id AS `reply_to_id: Simple`,
                p.created_at AS `created_at: chrono::NaiveDateTime`,
                p.deleted_at AS `deleted_at: chrono::NaiveDateTime`,
                (SELECT COUNT(*) FROM posts p2 WHERE p2.deleted_at IS NULL AND p2.reply_to_id=p.id) AS `count_replies!: i64`,
                (SELECT COUNT(*) FROM posts p2 WHERE p2.deleted_at IS NULL AND p2.repost_of_id=p.id AND p2.content IS NULL) AS `count_reposts!: i64`,
                (SELECT COUNT(*) FROM posts p2 WHERE p2.deleted_at IS NULL AND p2.repost_of_id=p.id AND p2.content IS NOT NULL) AS `count_quotes!: i64`,
                IIF(? IS NULL, NULL, (SELECT COUNT(*)>0 FROM posts p2 WHERE p2.deleted_at IS NULL AND p2.repost_of_id=p.id AND p2.content IS NULL AND p2.poster_id=? LIMIT 1)) AS `reposted_by_you: bool`,
                IIF(? IS NULL, NULL, (SELECT COUNT(*)>0 FROM post_favorites pf WHERE pf.post_id=p.id AND pf.user_id=? AND NOT pf.is_bookmark LIMIT 1)) AS `favorited_by_you: bool`,
                IIF(? IS NULL, NULL, (SELECT COUNT(*)>0 FROM post_favorites pf WHERE pf.post_id=p.id AND pf.user_id=? AND pf.is_bookmark LIMIT 1)) AS `bookmarked_by_you: bool`,
                IIF(? IS NULL, NULL, (SELECT reaction_str FROM post_reactions pr WHERE pr.post_id=p.id AND pr.user_id=? LIMIT 1)) AS `reaction_str_by_you?: String`
            FROM posts p
            INNER JOIN users u ON p.poster_id = u.id
            WHERE p.id = ?
              AND (
                (p.poster_id = ?)
                OR (
                    p.privacy IN ('public', 'unlisted')
                    OR (? AND p.privacy = 'follower' AND EXISTS(
                        SELECT 1 FROM user_follows WHERE followee_id=p.poster_id AND follower_id=?
                    ))
                    OR (? AND p.privacy = 'private' AND EXISTS(
                        SELECT 1 FROM post_mentions WHERE post_id=p.id AND target_user_id=?
                    ))
                )
              )
            "#,
            viewer_id, viewer_id,
            viewer_id, viewer_id,
            viewer_id, viewer_id,
            viewer_id, viewer_id,
            post_id,
            viewer_id,
            viewer_id_is_valid,
            viewer_id,
            viewer_id_is_valid,
            viewer_id
        )
        .fetch_optional(&self.pool)
        .await?;

        if let Some(p) = post {
            let p_id_str = p.id.to_string();
            let reaction_count = sqlx::query_as!(
                ReactionCount,
                r#"
            SELECT r.reaction_str, COUNT(*) AS `count`
            FROM post_reactions r
            WHERE r.post_id=?
              AND r.reaction_str IS NOT NULL
            GROUP BY r.reaction_str
            "#,
                p_id_str
            )
            .fetch_all(&self.pool)
            .await?;
            let reaction_count = reaction_count
                .into_iter()
                .map(|r| PostReaction {
                    name: r.reaction_str.unwrap(),
                    count: r.count,
                })
                .collect();

            let p_id_str = p.id.to_string();
            let mentioned_users = sqlx::query_as!(
                PostMentionedUser,
                r#"
                    SELECT u.id AS `id: Simple`, u.uri AS `uri`, u.username, u.host, u.inbox
                    FROM post_mentions m
                    INNER JOIN users u ON m.target_user_id=u.id
                    WHERE m.post_id=?
                    "#,
                p_id_str
            )
            .fetch_all(&self.pool)
            .await?;

            let post_uri = self.id_getter.get_post_id(&UserPostAsPost(&p));
            let user_uri = self.id_getter.get_user_id(&UserPostAsAuthor(&p));

            let repost_of_uri = match p.repost_of_id {
                None => None,
                Some(repost_of_id) => {
                    let p = self
                        .fetch_single_post(&PostSpecifier::from_id(repost_of_id), &viewer)
                        .await?;
                    Some(self.id_getter.get_post_id(&p))
                }
            };
            let reply_to_uri = match p.reply_to_id {
                None => None,
                Some(reply_to_id) => {
                    let p = self
                        .fetch_single_post(&PostSpecifier::from_id(reply_to_id), &viewer)
                        .await?;
                    Some(self.id_getter.get_post_id(&p))
                }
            };

            Ok(UserPostEntryBuilder::default()
                .id(p.id)
                .uri(post_uri)
                .author(
                    PostAuthorBuilder::default()
                        .id(p.author_id)
                        .uri(user_uri)
                        .username(p.author_username)
                        .host(p.author_host)
                        .nickname(p.author_nickname)
                        .inbox(None)
                        .build()
                        .unwrap(),
                )
                .content(p.content)
                .privacy(p.privacy.parse().unwrap())
                .repost_of_id(p.repost_of_id)
                .repost_of_uri(repost_of_uri)
                .reply_to_id(p.reply_to_id)
                .reply_to_uri(reply_to_uri)
                .created_at(chrono::DateTime::from_naive_utc_and_offset(
                    p.created_at,
                    chrono::Utc,
                ))
                .deleted_at(
                    p.deleted_at
                        .map(|d| chrono::DateTime::from_naive_utc_and_offset(d, chrono::Utc)),
                )
                .counts(
                    PostCountsBuilder::default()
                        .reactions(reaction_count)
                        .replies(p.count_replies)
                        .reposts(p.count_reposts)
                        .quotes(p.count_quotes)
                        .build()
                        .unwrap(),
                )
                .reposted_by_you(p.reposted_by_you)
                .favorited_by_you(p.favorited_by_you)
                .bookmarked_by_you(p.bookmarked_by_you)
                .mentioned_users(mentioned_users)
                .reaction_str_by_you(p.reaction_str_by_you)
                .build()
                .unwrap())
        } else {
            return Err(PostFetchError::PostNotFound.into());
        }
    }

    // async fn delete_single_post(
    //     &mut self,
    //     post: &PostSpecifier,
    //     viewer: &UserSpecifier,
    // ) -> Result<(), anyhow::Error> {
    //     let post = self.fetch_single_post(post, &Some(viewer.clone())).await;
    //     let post = match post {
    //         Ok(p) => p,
    //         Err(e) => match e.downcast::<PostFetchError>() {
    //             Ok(PostFetchError::PostNotFound) => {
    //                 return Err(PostDeleteError::PostNotFound.into())
    //             }
    //             Err(e) => return Err(e),
    //         },
    //     };

    //     // check if viewer is the author
    //     let viewer_user = self.finder.find_user_by_specifier(viewer).await?;
    //     if *post.author().id() != viewer_user.id {
    //         return Err(PostDeleteError::Unauthorized.into());
    //     }

    //     let mut tx = self.pool.begin().await?;

    //     let post_id = post.id();
    //     sqlx::query!(
    //         r#"
    //         UPDATE posts SET deleted_at=CURRENT_TIMESTAMP WHERE id=?
    //         "#,
    //         post_id.to_string()
    //     )
    //     .execute(&mut *tx)
    //     .await?;

    //     tx.commit().await?;

    //     Ok(())
    // }

    async fn fetch_user_posts(
        &mut self,
        user: &UserSpecifier,
        viewer: &Option<UserSpecifier>,
        options: &FetchUserPostsOptions,
    ) -> Result<Vec<UserPostEntry>, anyhow::Error> {
        let user = self.finder.find_user_by_specifier(user).await?;
        let viewer_u = if let Some(viewer) = viewer {
            Some(self.finder.find_user_by_specifier(viewer).await?)
        } else {
            None
        };

        let (before_date_valid, before_date) = (
            options.before_date.is_some(),
            options.before_date.map(|d| d.naive_utc()),
        );

        let posts = match viewer_u {
            None => {
                let user_id_str = user.id.to_string();
                sqlx::query_as!(
                    UserPost,
                    r#"
                    SELECT
                        p.id AS `id: Simple`,
                        p.uri AS `uri`,
                        u.id `author_id: Simple`,
                        u.uri AS `author_uri`,
                        u.username AS `author_username`,
                        u.host AS `author_host`,
                        u.nickname AS `author_nickname`,
                        p.content,
                        p.privacy,
                        p.repost_of_id AS `repost_of_id: Simple`,
                        p.reply_to_id AS `reply_to_id: Simple`,
                        p.created_at AS `created_at: chrono::NaiveDateTime`,
                        p.deleted_at AS `deleted_at: chrono::NaiveDateTime`,
                        (SELECT COUNT(*) FROM posts p2 WHERE p2.deleted_at IS NULL AND p2.reply_to_id=p.id) AS `count_replies!: i64`,
                        (SELECT COUNT(*) FROM posts p2 WHERE p2.deleted_at IS NULL AND p2.repost_of_id=p.id AND p2.content IS NULL) AS `count_reposts!: i64`,
                        (SELECT COUNT(*) FROM posts p2 WHERE p2.deleted_at IS NULL AND p2.repost_of_id=p.id AND p2.content IS NOT NULL) AS `count_quotes!: i64`,
                        NULL AS `reposted_by_you: bool`,
                        NULL AS `favorited_by_you: bool`,
                        NULL AS `bookmarked_by_you: bool`,
                        NULL AS `reaction_str_by_you?: String`
                    FROM posts p
                    INNER JOIN users u ON p.poster_id = u.id
                    WHERE p.poster_id=?
                      AND p.privacy IN ('public', 'unlisted')
                      AND (NOT ? OR p.created_at <= ?)
                      AND (? OR p.deleted_at IS NULL)
                    ORDER BY p.created_at DESC
                    LIMIT ?
                    "#,
                    user_id_str,
                    before_date_valid,
                    before_date,
                    options.include_deleted,
                    options.limit,
                )
                .fetch_all(&self.pool)
                .await?
            }
            Some(viewer) => {
                let user_id_str = user.id.to_string();
                let viewer_id_str = viewer.id.to_string();
                sqlx::query_as!(
                    UserPost,
                    r#"
                    SELECT
                        p.id AS `id: Simple`,
                        p.uri AS `uri`,
                        u.id `author_id: Simple`,
                        u.uri AS `author_uri`,
                        u.username AS `author_username`,
                        u.host AS `author_host`,
                        u.nickname AS `author_nickname`,
                        p.content,
                        p.privacy,
                        p.repost_of_id AS `repost_of_id: Simple`,
                        p.reply_to_id AS `reply_to_id: Simple`,
                        p.created_at AS `created_at: chrono::NaiveDateTime`,
                        p.deleted_at AS `deleted_at: chrono::NaiveDateTime`,
                        (SELECT COUNT(*) FROM posts p2 WHERE p2.deleted_at IS NULL AND p2.reply_to_id=p.id) AS `count_replies!: i64`,
                        (SELECT COUNT(*) FROM posts p2 WHERE p2.deleted_at IS NULL AND p2.repost_of_id=p.id AND p2.content IS NULL) AS `count_reposts!: i64`,
                        (SELECT COUNT(*) FROM posts p2 WHERE p2.deleted_at IS NULL AND p2.repost_of_id=p.id AND p2.content IS NOT NULL) AS `count_quotes!: i64`,
                        (SELECT COUNT(*)>0 FROM posts p2 WHERE p2.deleted_at IS NULL AND p2.repost_of_id=p.id AND p2.content IS NULL AND p2.poster_id=? LIMIT 1) AS `reposted_by_you: bool`,
                        (SELECT COUNT(*)>0 FROM post_favorites pf WHERE pf.post_id=p.id AND pf.user_id=? AND NOT pf.is_bookmark LIMIT 1) AS `favorited_by_you: bool`,
                        (SELECT COUNT(*)>0 FROM post_favorites pf WHERE pf.post_id=p.id AND pf.user_id=? AND pf.is_bookmark LIMIT 1) AS `bookmarked_by_you: bool`,
                        (SELECT reaction_str FROM post_reactions pr WHERE pr.post_id=p.id AND pr.user_id=? LIMIT 1) AS `reaction_str_by_you?: String`
                    FROM posts p
                    INNER JOIN users u ON p.poster_id = u.id
                    WHERE p.poster_id=?
                      AND (
                        p.poster_id=?
                        OR p.privacy IN ('public', 'unlisted')
                        OR (p.privacy = 'follower' AND EXISTS(SELECT 1 FROM user_follows WHERE followee_id=? AND follower_id=?))
                        OR (p.privacy = 'private' AND EXISTS(SELECT 1 FROM post_mentions WHERE post_id=p.id AND target_user_id=?))
                      )
                      AND (NOT ? OR p.created_at <= ?)
                      AND (? OR p.deleted_at IS NULL)
                    ORDER BY p.created_at DESC
                    LIMIT ?
                    "#,
                    viewer_id_str,
                    viewer_id_str,
                    viewer_id_str,
                    viewer_id_str,
                    user_id_str,
                    viewer_id_str,
                    user_id_str,
                    viewer_id_str,
                    viewer_id_str,
                    before_date_valid,
                    before_date,
                    options.include_deleted,
                    options.limit,
                )
                .fetch_all(&self.pool)
                .await?
            }
        };

        let mut entries = Vec::new();
        for p in posts {
            let p_id_str = p.id.to_string();
            let reaction_count = sqlx::query_as!(
                ReactionCount,
                r#"
            SELECT r.reaction_str, COUNT(*) AS `count`
            FROM post_reactions r
            WHERE r.post_id=?
              AND r.reaction_str IS NOT NULL
            GROUP BY r.reaction_str
            "#,
                p_id_str
            )
            .fetch_all(&self.pool)
            .await?;
            let reaction_count = reaction_count
                .into_iter()
                .map(|r| PostReaction {
                    name: r.reaction_str.unwrap(),
                    count: r.count,
                })
                .collect();

            let mentioned_users = sqlx::query_as!(
                PostMentionedUser,
                r#"
                SELECT u.id AS `id: Simple`, u.uri AS `uri`, u.username, u.host, u.inbox
                FROM post_mentions m
                INNER JOIN users u ON m.target_user_id=u.id
                WHERE m.post_id=?
                "#,
                p_id_str
            )
            .fetch_all(&self.pool)
            .await?;

            let entry = {
                let post_uri = self.id_getter.get_post_id(&UserPostAsPost(&p));
                let user_uri = self.id_getter.get_user_id(&UserPostAsAuthor(&p));

                let repost_of_uri = match p.repost_of_id {
                    None => None,
                    Some(repost_of_id) => {
                        let p = self
                            .fetch_single_post(&PostSpecifier::from_id(repost_of_id), &viewer)
                            .await?;
                        Some(self.id_getter.get_post_id(&p))
                    }
                };
                let reply_to_uri = match p.reply_to_id {
                    None => None,
                    Some(reply_to_id) => {
                        let p = self
                            .fetch_single_post(&PostSpecifier::from_id(reply_to_id), &viewer)
                            .await?;
                        Some(self.id_getter.get_post_id(&p))
                    }
                };

                UserPostEntryBuilder::default()
                    .id(p.id)
                    .uri(post_uri)
                    .author(
                        PostAuthorBuilder::default()
                            .id(p.author_id)
                            .uri(user_uri)
                            .username(p.author_username)
                            .host(p.author_host)
                            .nickname(p.author_nickname)
                            .inbox(None)
                            .build()
                            .unwrap(),
                    )
                    .content(p.content)
                    .privacy(p.privacy.parse().unwrap())
                    .repost_of_id(p.repost_of_id)
                    .repost_of_uri(repost_of_uri)
                    .reply_to_id(p.reply_to_id)
                    .reply_to_uri(reply_to_uri)
                    .created_at(chrono::DateTime::from_naive_utc_and_offset(
                        p.created_at,
                        chrono::Utc,
                    ))
                    .deleted_at(
                        p.deleted_at
                            .map(|d| chrono::DateTime::from_naive_utc_and_offset(d, chrono::Utc)),
                    )
                    .counts(
                        PostCountsBuilder::default()
                            .reactions(reaction_count)
                            .replies(p.count_replies)
                            .reposts(p.count_reposts)
                            .quotes(p.count_quotes)
                            .build()
                            .unwrap(),
                    )
                    .reposted_by_you(p.reposted_by_you)
                    .favorited_by_you(p.favorited_by_you)
                    .bookmarked_by_you(p.bookmarked_by_you)
                    .mentioned_users(mentioned_users)
                    .reaction_str_by_you(p.reaction_str_by_you)
                    .build()
                    .unwrap()
            };
            entries.push(entry);
        }

        Ok(entries)
    }

    async fn fetch_timeline(
        &mut self,
        user_spec: &UserSpecifier,
        options: &TimelineOptions,
    ) -> Result<Vec<UserPostEntry>, anyhow::Error> {
        let user = self.finder.find_user_by_specifier(user_spec).await?;

        let (before_date_valid, before_date) = (
            options.before_date.is_some(),
            options.before_date.map(|d| d.naive_utc()),
        );

        let user_id_str = user.id.to_string();
        let posts = sqlx::query_as!(
            UserPost,
            r#"
            SELECT
                p.id AS `id: Simple`,
                p.uri AS `uri`,
                u.id `author_id: Simple`,
                u.uri AS `author_uri`,
                u.username AS `author_username`,
                u.host AS `author_host`,
                u.nickname AS `author_nickname`,
                p.content,
                p.privacy,
                p.repost_of_id AS `repost_of_id: Simple`,
                p.reply_to_id AS `reply_to_id: Simple`,
                p.created_at AS `created_at: chrono::NaiveDateTime`,
                p.deleted_at AS `deleted_at: chrono::NaiveDateTime`,
                (SELECT COUNT(*) FROM posts p2 WHERE p2.deleted_at IS NULL AND p2.reply_to_id=p.id) AS `count_replies!: i64`,
                (SELECT COUNT(*) FROM posts p2 WHERE p2.deleted_at IS NULL AND p2.repost_of_id=p.id AND p2.content IS NULL) AS `count_reposts!: i64`,
                (SELECT COUNT(*) FROM posts p2 WHERE p2.deleted_at IS NULL AND p2.repost_of_id=p.id AND p2.content IS NOT NULL) AS `count_quotes!: i64`,
                (SELECT COUNT(*)>0 FROM posts p2 WHERE p2.deleted_at IS NULL AND p2.repost_of_id=p.id AND p2.content IS NULL AND p2.poster_id=? LIMIT 1) AS `reposted_by_you: bool`,
                (SELECT COUNT(*)>0 FROM post_favorites pf WHERE pf.post_id=p.id AND pf.user_id=? AND NOT pf.is_bookmark LIMIT 1) AS `favorited_by_you: bool`,
                (SELECT COUNT(*)>0 FROM post_favorites pf WHERE pf.post_id=p.id AND pf.user_id=? AND pf.is_bookmark LIMIT 1) AS `bookmarked_by_you: bool`,
                (SELECT reaction_str FROM post_reactions pr WHERE pr.post_id=p.id AND pr.user_id=? LIMIT 1) AS `reaction_str_by_you?: String`
            FROM posts p
            INNER JOIN users u ON p.poster_id = u.id
            WHERE (
                p.poster_id=?
                OR (? AND p.privacy = 'public')
                OR (p.privacy IN ('public', 'unlisted', 'follower') AND EXISTS(SELECT 1 FROM user_follows WHERE followee_id=p.poster_id AND follower_id=?))
                OR (EXISTS(SELECT 1 FROM post_mentions WHERE post_id=p.id AND target_user_id=?))
                OR (EXISTS(SELECT 1 FROM posts p2 WHERE p2.poster_id=? AND p2.id=p.reply_to_id))
              )
              AND (NOT ? OR p.created_at <= ?)
              AND deleted_at IS NULL
            ORDER BY p.created_at DESC
            LIMIT ?
            "#,
            user_id_str,
            user_id_str,
            user_id_str,
            user_id_str,
            user_id_str,
            options.include_all_public,
            user_id_str,
            user_id_str,
            user_id_str,
            before_date_valid,
            before_date,
            options.limit,
        )
        .fetch_all(&self.pool)
        .await?;

        let mut entries = Vec::new();
        for p in posts {
            let p_id_str = p.id.to_string();
            let reaction_count = sqlx::query_as!(
                ReactionCount,
                r#"
            SELECT r.reaction_str, COUNT(*) AS `count`
            FROM post_reactions r
            WHERE r.post_id=?
              AND r.reaction_str IS NOT NULL
            GROUP BY r.reaction_str
            "#,
                p_id_str
            )
            .fetch_all(&self.pool)
            .await?;
            let reaction_count = reaction_count
                .into_iter()
                .map(|r| PostReaction {
                    name: r.reaction_str.unwrap(),
                    count: r.count,
                })
                .collect();

            let mentioned_users = sqlx::query_as!(
                PostMentionedUser,
                r#"
                    SELECT u.id AS `id: Simple`, u.uri AS `uri`, u.username, u.host, u.inbox
                    FROM post_mentions m
                    INNER JOIN users u ON m.target_user_id=u.id
                    WHERE m.post_id=?
                    "#,
                p_id_str
            )
            .fetch_all(&self.pool)
            .await?;

            let post_uri = self.id_getter.get_post_id(&UserPostAsPost(&p));
            let user_uri = self.id_getter.get_user_id(&UserPostAsAuthor(&p));

            let repost_of_uri = match p.repost_of_id {
                None => None,
                Some(repost_of_id) => {
                    let p = self
                        .fetch_single_post(
                            &PostSpecifier::from_id(repost_of_id),
                            &Some(user_spec.clone()),
                        )
                        .await?;
                    Some(self.id_getter.get_post_id(&p))
                }
            };
            let reply_to_uri = match p.reply_to_id {
                None => None,
                Some(reply_to_id) => {
                    let p = self
                        .fetch_single_post(
                            &PostSpecifier::from_id(reply_to_id),
                            &Some(user_spec.clone()),
                        )
                        .await?;
                    Some(self.id_getter.get_post_id(&p))
                }
            };

            let entry = UserPostEntryBuilder::default()
                .id(p.id)
                .uri(post_uri)
                .author(
                    PostAuthorBuilder::default()
                        .id(p.author_id)
                        .uri(user_uri)
                        .username(p.author_username)
                        .host(p.author_host)
                        .nickname(p.author_nickname)
                        .inbox(None)
                        .build()
                        .unwrap(),
                )
                .content(p.content)
                .privacy(p.privacy.parse().unwrap())
                .repost_of_id(p.repost_of_id)
                .repost_of_uri(repost_of_uri)
                .reply_to_id(p.reply_to_id)
                .reply_to_uri(reply_to_uri)
                .created_at(chrono::DateTime::from_naive_utc_and_offset(
                    p.created_at,
                    chrono::Utc,
                ))
                .deleted_at(
                    p.deleted_at
                        .map(|d| chrono::DateTime::from_naive_utc_and_offset(d, chrono::Utc)),
                )
                .counts(
                    PostCountsBuilder::default()
                        .reactions(reaction_count)
                        .replies(p.count_replies)
                        .reposts(p.count_reposts)
                        .quotes(p.count_quotes)
                        .build()
                        .unwrap(),
                )
                .reposted_by_you(p.reposted_by_you)
                .favorited_by_you(p.favorited_by_you)
                .bookmarked_by_you(p.bookmarked_by_you)
                .mentioned_users(mentioned_users)
                .reaction_str_by_you(p.reaction_str_by_you)
                .build()
                .unwrap();
            entries.push(entry);
        }

        Ok(entries)
    }
}
