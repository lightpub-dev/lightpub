use std::collections::HashSet;

use async_recursion::async_recursion;
use async_trait::async_trait;
use derive_more::Constructor;
use sqlx::MySqlPool;
use tracing::warn;
use uuid::{fmt::Simple, Uuid};

use crate::{
    holder,
    models::{
        api_response::{PostAuthorBuilder, PostCountsBuilder, UserPostEntry, UserPostEntryBuilder},
        apub::CreatableObject,
        ApubRenderablePost, HasRemoteUri, PostPrivacy,
    },
    services::{
        apub::{
            post::PostContentService,
            render::{ApubRendererService, TargetedUser},
        },
        id::IDGetterService,
        AllUserFinderService, ApubRequestService, FetchUserPostsOptions, PostCreateError,
        PostCreateRequest, PostCreateService, ServiceError, SignerService, TimelineOptions,
        UserPostService,
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
        depth: i32,
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
                        .fetch_post_id(
                            &PostSpecifier::from_id(local_post_id),
                            not_found_err,
                            depth + 1,
                        )
                        .await;
                }

                let result = sqlx::query!("SELECT id AS `id: Simple` FROM posts WHERE uri=?", uri)
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
                let CreatableObject::Note(remote_post) = remote_post;
                self.create_post_(&remote_post.try_into().unwrap(), depth + 1)
                    .await
            }
        }
    }

    async fn create_post_(
        &mut self,
        req: &crate::services::PostCreateRequest,
        depth: i32,
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
                .fetch_post_id(&s, PostCreateError::RepostOfNotFound, depth + 1)
                .await?
                .into(),
        };

        let reply_to_id = match reply_to_spec {
            None => None,
            Some(s) => self
                .fetch_post_id(&s, PostCreateError::ReplyToNotFound, depth + 1)
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

#[async_trait]
impl PostCreateService for DBPostCreateService {
    async fn create_post(
        &mut self,
        req: &crate::services::PostCreateRequest,
    ) -> Result<Simple, crate::services::ServiceError<crate::services::PostCreateError>> {
        self.create_post_(req, 0).await
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

#[derive(Constructor)]
pub struct DBUserPostService {
    pool: MySqlPool,
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
    count_replies: i64,
    count_reposts: i64,
    count_quotes: i64,
    reposted_by_you: Option<bool>,
    favorited_by_you: Option<bool>,
    bookmarked_by_you: Option<bool>,
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

#[async_trait]
impl UserPostService for DBUserPostService {
    async fn fetch_user_posts(
        &mut self,
        user: &UserSpecifier,
        viewer: &Option<UserSpecifier>,
        options: &FetchUserPostsOptions,
    ) -> Result<Vec<UserPostEntry>, anyhow::Error> {
        let user = self.finder.find_user_by_specifier(user).await?;
        let viewer = if let Some(viewer) = viewer {
            Some(self.finder.find_user_by_specifier(viewer).await?)
        } else {
            None
        };

        let (before_date_valid, before_date) = (
            options.before_date.is_some(),
            options.before_date.map(|d| d.naive_utc()),
        );

        let posts = match viewer {
            None => {
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
                        p.created_at,
                        0 AS `count_replies`,
                        0 AS `count_reposts`,
                        0 AS `count_quotes`,
                        NULL AS `reposted_by_you: bool`,
                        NULL AS `favorited_by_you: bool`,
                        NULL AS `bookmarked_by_you: bool`
                    FROM posts p
                    INNER JOIN users u ON p.poster_id = u.id
                    WHERE p.poster_id=?
                      AND p.privacy IN ('public', 'unlisted')
                      AND (NOT ? OR p.created_at <= ?)
                    ORDER BY p.created_at DESC
                    LIMIT ?
                    "#,
                    user.id.to_string(),
                    before_date_valid,
                    before_date,
                    options.limit,
                )
                .fetch_all(&self.pool)
                .await?
            }
            Some(viewer) => {
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
                        p.created_at,
                        0 AS `count_replies`,
                        0 AS `count_reposts`,
                        0 AS `count_quotes`,
                        NULL AS `reposted_by_you: bool`,
                        NULL AS `favorited_by_you: bool`,
                        NULL AS `bookmarked_by_you: bool`
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
                    ORDER BY p.created_at DESC
                    LIMIT ?
                    "#,
                    user.id.to_string(),
                    viewer.id.to_string(),
                    user.id.to_string(),
                    viewer.id.to_string(),
                    viewer.id.to_string(),
                    before_date_valid,
                    before_date,
                    options.limit,
                )
                .fetch_all(&self.pool)
                .await?
            }
        };

        let entries = posts
            .into_iter()
            .map(|p| {
                let post_uri = self.id_getter.get_post_id(&UserPostAsPost(&p));
                let user_uri = self.id_getter.get_user_id(&UserPostAsAuthor(&p));
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
                            .build()
                            .unwrap(),
                    )
                    .content(p.content)
                    .privacy(p.privacy.parse().unwrap())
                    .repost_of_id(p.repost_of_id)
                    .reply_to_id(p.reply_to_id)
                    .created_at(chrono::DateTime::from_naive_utc_and_offset(
                        p.created_at,
                        chrono::Utc,
                    ))
                    .counts(
                        PostCountsBuilder::default()
                            .reactions(vec![])
                            .replies(p.count_replies)
                            .reposts(p.count_reposts)
                            .quotes(p.count_quotes)
                            .build()
                            .unwrap(),
                    )
                    .reposted_by_you(p.reposted_by_you)
                    .favorited_by_you(p.favorited_by_you)
                    .bookmarked_by_you(p.bookmarked_by_you)
                    .build()
                    .unwrap()
            })
            .collect();

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
                p.created_at,
                0 AS `count_replies`,
                0 AS `count_reposts`,
                0 AS `count_quotes`,
                NULL AS `reposted_by_you: bool`,
                NULL AS `favorited_by_you: bool`,
                NULL AS `bookmarked_by_you: bool`
            FROM posts p
            INNER JOIN users u ON p.poster_id = u.id
            WHERE (
                p.poster_id=?
                OR (? AND p.privacy = 'public')
                OR (p.privacy IN ('public', 'unlisted', 'follower') AND EXISTS(SELECT 1 FROM user_follows WHERE followee_id=p.poster_id AND follower_id=?))
                OR (p.privacy = 'private' AND EXISTS(SELECT 1 FROM post_mentions WHERE post_id=p.id AND target_user_id=?))
              )
              AND (NOT ? OR p.created_at <= ?)
            ORDER BY p.created_at DESC
            LIMIT ?
            "#,
            user.id.to_string(),
            options.include_all_public,
            user.id.to_string(),
            user.id.to_string(),
            before_date_valid,
            before_date,
            options.limit,
        )
        .fetch_all(&self.pool)
        .await?;

        let entries = posts
            .into_iter()
            .map(|p| {
                let post_uri = self.id_getter.get_post_id(&UserPostAsPost(&p));
                let user_uri = self.id_getter.get_user_id(&UserPostAsAuthor(&p));
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
                            .build()
                            .unwrap(),
                    )
                    .content(p.content)
                    .privacy(p.privacy.parse().unwrap())
                    .repost_of_id(p.repost_of_id)
                    .reply_to_id(p.reply_to_id)
                    .created_at(chrono::DateTime::from_naive_utc_and_offset(
                        p.created_at,
                        chrono::Utc,
                    ))
                    .counts(
                        PostCountsBuilder::default()
                            .reactions(vec![])
                            .replies(p.count_replies)
                            .reposts(p.count_reposts)
                            .quotes(p.count_quotes)
                            .build()
                            .unwrap(),
                    )
                    .reposted_by_you(p.reposted_by_you)
                    .favorited_by_you(p.favorited_by_you)
                    .bookmarked_by_you(p.bookmarked_by_you)
                    .build()
                    .unwrap()
            })
            .collect();

        Ok(entries)
    }
}
