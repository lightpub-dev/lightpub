/*
Lightpub: a simple ActivityPub server
Copyright (C) 2025 tinaxd

This program is free software: you can redistribute it and/or modify
it under the terms of the GNU Affero General Public License as published
by the Free Software Foundation, either version 3 of the License, or
(at your option) any later version.

This program is distributed in the hope that it will be useful,
but WITHOUT ANY WARRANTY; without even the implied warranty of
MERCHANTABILITY or FITNESS FOR A PARTICULAR PURPOSE.  See the
GNU Affero General Public License for more details.

You should have received a copy of the GNU Affero General Public License
along with this program.  If not, see <https://www.gnu.org/licenses/>.
*/

use std::borrow::Cow;

use activitypub_federation::config::Data;
use activitypub_federation::fetch::object_id::ObjectId;
use actix_web::http::StatusCode;
use apub::OutboxActivity;
use chrono::{DateTime, Utc};
use create::ExistingNote;
use create::NoteCreateError;
use create::upsert_note;
use entity::sea_orm_active_enums::Visibility;
use expected_error_derive::ExpectedError;
use get::get_apubnote_by_id;
use get::get_url_of_note_model;
use migration::Alias;
use migration::Expr;
use migration::ExprTrait;
use migration::Query;
use nestify::nest;
use renderer::NoteRenderer;
use sea_orm::ActiveEnum;
use sea_orm::ColumnTrait;
use sea_orm::Condition;
use sea_orm::ConnectionTrait;
use sea_orm::Order;
use sea_orm::QueryFilter;
use sea_orm::QueryOrder;
use sea_orm::QuerySelect;
use sea_orm::Set;
use sea_orm::StatementBuilder;
use sea_orm::{ActiveModelTrait, EntityTrait};
use serde::Deserialize;
use serde::Serialize;
use thiserror::Error;
use upload::NoteUploadModel;
use url::Url;

use crate::MyFederationData;
use crate::services::MapToUnknown;
use crate::services::{ServiceError, id::Identifier};
use crate::utils::sanitize::CleanString;

use super::apub::AnnounceActivity;
use super::create_error_simple;
use super::db::MaybeTxConn;
use super::kv::KVObject;
use super::notification::NotificationBody;
use super::notification::add_notification;
use super::queue::QConn;
use super::timeline::get_note_reply_ids;
use super::timeline::get_timeline_note_ids;
use super::user::SimpleUserModel;
use super::user::get_apubuser_by_id;
use super::user::get_user_by_id;
use super::{
    ServiceResult,
    db::Conn,
    id::{NoteID, UserID},
    user::UserSpecifier,
};

mod apub;
mod cache;
mod count;
mod create;
mod delete;
mod get;
pub mod hashtag;
mod like;
pub mod mention;
pub mod renderer;
mod upload;
mod visibility;

pub use apub::{
    ApubNoteAttachment, ApubNoteModel, ApubNoteSourceModel, ApubTagType, CalculateToAndCcResult,
    NoteApubData, NoteApubHashtagData, NoteApubMentionData, NoteWithApubModel, calculate_to_and_cc,
    calculate_to_and_cc_of_renote,
};
pub use cache::invalidate_note_basic_cache;
pub use count::count_local_notes;
pub use create::{
    NoteUpload, PostCreateOptions, PostCreateOptionsBuilder, PostCreateOptionsBuilderError,
};
pub use delete::{delete_note_by_id, delete_note_by_id_, delete_renote_by_id};
pub use get::{
    get_apubnote_by_id_visibility_check, get_note_by_id, get_note_by_id_visibility_check,
    get_note_by_spec,
};
pub use like::{note_like_add, note_like_remove};
pub use upload::NoteUploadModelData;
pub use visibility::{VisibilityModel, note_visibility_check};

#[derive(Debug, Clone, Copy, PartialEq, Eq, Serialize, Deserialize)]
pub enum ContentType {
    #[serde(rename = "plain")]
    Plain,
    #[serde(rename = "md")]
    Md,
    #[serde(rename = "html")]
    Html,
    #[serde(rename = "latex")]
    Latex,
}

impl ContentType {
    pub fn as_str(&self) -> &'static str {
        match self {
            ContentType::Plain => "plain",
            ContentType::Md => "md",
            ContentType::Html => "html",
            ContentType::Latex => "latex",
        }
    }

    pub fn from_str(s: &str) -> Option<Self> {
        match s {
            "plain" => Some(ContentType::Plain),
            "md" => Some(ContentType::Md),
            "html" => Some(ContentType::Html),
            "latex" => Some(ContentType::Latex),
            _ => None,
        }
    }
}

#[derive(Debug, Clone)]
pub enum Mention {
    ByURL(Url),
    ByUsername(String, Option<String>),
}

impl Mention {
    pub fn into_user_spec(self) -> UserSpecifier {
        use Mention::*;
        match self {
            ByURL(url) => UserSpecifier::URL(url),
            ByUsername(username, domain) => UserSpecifier::Username(username, domain),
        }
    }
}

pub async fn create_renote(
    conn: &Conn,
    rconn: &KVObject,
    qconn: &QConn,
    user_id: UserID,
    target_note_id: NoteID,
    visibility: VisibilityModel,
    base_url: &Url,
) -> ServiceResult<NoteID> {
    if visibility == VisibilityModel::Follower || visibility == VisibilityModel::Private {
        return Err(ServiceError::known(
            NoteCreateError::InvalidRenoteVisibility,
        ));
    }

    let tx = conn.as_tx().await?.into();

    let user = get_apubuser_by_id(&tx, user_id, base_url).await?;
    let user = match user {
        None => return Err(ServiceError::known(NoteCreateError::AuthorNotFound)),
        Some(u) => u,
    };

    // check if already renote is created
    let existing_renote = entity::note::Entity::find()
        .filter(
            Condition::all()
                .add(entity::note::Column::AuthorId.eq(user_id.as_db()))
                .add(entity::note::Column::RenoteOfId.eq(target_note_id.as_db()))
                .add(entity::note::Column::Content.is_null())
                .add(entity::note::Column::DeletedAt.is_null()), // この条件はなくてもいいはず
        )
        .one(&tx)
        .await
        .map_err_unknown()?;
    if existing_renote.is_some() {
        return Err(ServiceError::known(NoteCreateError::AlreadyRenoted));
    }

    let target_note =
        get_note_by_id_visibility_check(&tx, rconn, target_note_id, Some(user_id), false)
            .await?
            .ok_or(ServiceError::known(NoteCreateError::RenoteTargetNotFound))?;
    if target_note.basic.content.is_none() {
        return Err(ServiceError::known(NoteCreateError::RenoteTargetNotFound));
    }

    if matches!(
        target_note.basic.visibility,
        VisibilityModel::Private | VisibilityModel::Follower
    ) {
        return Err(ServiceError::known(NoteCreateError::RenoteNotAllowed));
    }

    let renote_id = NoteID::new_random();

    let now_time = Utc::now();

    let new_note = entity::note::ActiveModel {
        id: Set(renote_id.as_db()),
        author_id: Set(user_id.as_db()),
        content: Set(None),
        content_type: Set(None),
        created_at: Set(now_time.naive_utc()),
        inserted_at: Set(now_time.naive_utc()),
        visibility: Set(visibility.as_db()),
        renote_of_id: Set(Some(target_note_id.as_db())),
        ..Default::default()
    };
    let new_note = new_note.insert(&tx).await.map_err_unknown()?;

    // notification
    if target_note.basic.author.is_local() {
        let body = NotificationBody::Renoted(user_id, target_note_id);
        add_notification(&tx, target_note.basic.author.id, &body).await?;
    }

    // send activitypub note
    if user.is_local() {
        let target_note_apub = get_apubnote_by_id(&tx, target_note_id, base_url, false)
            .await?
            .expect("renote target should exist");
        let target_note_author = get_user_by_id(&tx, rconn, target_note_apub.basic.author.id)
            .await?
            .expect("renote target author should exist");
        let renote_apub_url = get_url_of_note_model(&new_note, base_url);
        let CalculateToAndCcResult { to, cc, inboxes } = calculate_to_and_cc_of_renote(
            &tx,
            user.basic.id,
            target_note_author.id,
            visibility,
            base_url,
        )
        .await?;
        let announce = AnnounceActivity::from_note(
            ObjectId::from(target_note_apub.apub.url.clone()),
            renote_apub_url,
            ObjectId::from(user.apub.url.clone()),
            to.into_iter().map(|u| ObjectId::from(u)).collect(),
            cc.into_iter().map(|u| ObjectId::from(u)).collect(),
            new_note.created_at.and_utc(),
        );
        qconn.queue_activity(announce, user, inboxes).await?;
    }

    tx.commit().await?;

    Ok(renote_id)
}

pub async fn create_note(
    conn: &Conn,
    rconn: &KVObject,
    qconn: &QConn,
    author_id: UserID,
    content: &str,
    content_type: ContentType,
    visibility: VisibilityModel,
    options: &PostCreateOptions,
    my_domain: &str,
    base_url: &Url,
    fed_data: &Data<MyFederationData>,
) -> ServiceResult<NoteID> {
    upsert_note(
        conn,
        rconn,
        qconn,
        None,
        author_id,
        content,
        content_type,
        Some(visibility),
        options,
        my_domain,
        base_url,
        fed_data,
    )
    .await
}

pub async fn edit_note(
    conn: &Conn,
    rconn: &KVObject,
    qconn: &QConn,
    editor_id: UserID,
    note_id: NoteID,
    content: &str,
    content_type: ContentType,
    hashtag_override: Option<Vec<String>>,
    mention_override: Option<Vec<Mention>>,
    my_domain: &str,
    base_url: &Url,
    fed_data: &Data<MyFederationData>,
) -> ServiceResult<()> {
    upsert_note(
        conn,
        rconn,
        qconn,
        Some(ExistingNote::ByID(note_id)),
        editor_id,
        content,
        content_type,
        None,
        &PostCreateOptionsBuilder::default()
            .hashtags_override(hashtag_override)
            .mentions_override(mention_override)
            .build()
            .unwrap(),
        my_domain,
        base_url,
        fed_data,
    )
    .await?;
    Ok(())
}

#[derive(Debug, Clone, Error, ExpectedError)]
pub enum NoteUrlError {
    #[error("No domain")]
    #[ee(status(StatusCode::BAD_REQUEST))]
    NoDomain,
    #[error("Bad URL")]
    #[ee(status(StatusCode::BAD_REQUEST))]
    BadURL,
}

#[derive(Debug, Clone)]
pub enum NoteSpecifier {
    ID(NoteID),
    URL(Url),
}

impl NoteSpecifier {
    pub fn url(url: impl Into<Url>) -> Self {
        Self::URL(url.into())
    }

    pub fn local_url_convert(&self, my_domain: &str) -> ServiceResult<Cow<Self>> {
        match self {
            Self::ID(_) => Ok(Cow::Borrowed(self)),
            Self::URL(url) => {
                if url
                    .domain()
                    .ok_or(ServiceError::known(NoteUrlError::NoDomain))?
                    == my_domain
                {
                    let path: Vec<_> = url.path().split("/").collect();
                    if path.len() != 3 {
                        return Err(ServiceError::known(NoteUrlError::BadURL));
                    }
                    if path[1] != "note" {
                        return Err(ServiceError::known(NoteUrlError::BadURL));
                    }
                    let note_id = NoteID::from_string(path[2])
                        .ok_or(ServiceError::known(NoteUrlError::BadURL))?;
                    Ok(Cow::Owned(NoteSpecifier::ID(note_id)))
                } else {
                    Ok(Cow::Borrowed(self))
                }
            }
        }
    }

    pub fn is_locally_stored(&self, my_domain: &str) -> bool {
        match self {
            Self::ID(_) => true,
            Self::URL(url) => url.domain().is_some_and(|d| d == my_domain),
        }
    }
}

#[derive(Debug, Clone)]
struct ApubDataWithInboxes {
    data: NoteApubData,
    inboxes: Vec<Url>,
}

nest! {
    #[derive(Debug, Clone, Serialize, Deserialize)]*
    #[serde(rename_all = "camelCase")]*
    pub struct BasicNoteModel {
        pub id: NoteID,
        pub author: pub struct NoteAuthorModel {
            pub id: UserID,
            pub username: String,
            pub nickname: String,
            pub domain: Option<String>,
        },
        pub content: Option<NoteContentModel>,
        pub visibility: VisibilityModel,
        pub created_at: DateTime<Utc>,
        pub updated_at: Option<DateTime<Utc>>,

        pub reply_to_id: Option<NoteID>,
        pub renote_of_id: Option<NoteID>,
        pub deleted_at: Option<DateTime<Utc>>,

        pub sensitive: bool,
        pub uploads: Vec<NoteUploadModel>,

    }
}

impl BasicNoteModel {
    pub fn is_renotable(&self) -> bool {
        self.visibility.is_renotable()
    }
}

nest! {
    #[derive(Debug, Clone, Serialize, Deserialize)]*
    #[serde(rename_all = "camelCase")]*
    pub struct DetailedNoteModel {
        pub basic: BasicNoteModel,
        pub details: pub struct NoteModelDetails {
            pub reply_count: u64,
            pub renote_count: u64,
            pub like_count: u64,

            pub renoted: Option<bool>,
            pub liked: Option<bool>,
            pub bookmarked: Option<bool>,

            pub hashtags: Vec<String>,
            pub mentions: Vec<pub struct NoteMentionModel {
                pub id: UserID,
                pub username: String,
                pub nickname: String,
                pub domain: Option<String>,
            }>,
        }
    }
}

impl NoteAuthorModel {
    pub fn is_remote(&self) -> bool {
        self.domain.is_some()
    }

    pub fn is_local(&self) -> bool {
        self.domain.is_none()
    }

    pub fn specifier(&self) -> String {
        UserSpecifier::username_and_domain_opt(self.username.clone(), self.domain.clone())
            .to_string()
    }
}

#[derive(Debug, Clone, Serialize, Deserialize, PartialEq)]
#[serde(rename_all = "camelCase")]
pub enum NoteContentModel {
    Plain(String),
    Md(String),
    Html(String),
    Latex(String),
}

impl NoteContentModel {
    pub async fn render_to_html(&self) -> CleanString {
        match self {
            NoteContentModel::Plain(c) => renderer::PlainNoteRenderer::new().render_note(c).await,
            NoteContentModel::Md(c) => renderer::MdNoteRenderer::new().render_note(c).await,
            NoteContentModel::Html(c) => renderer::HtmlNoteRenderer::new().render_note(c).await,
            _ => todo!("note renderer"),
        }
    }

    pub fn as_raw_text(&self) -> &str {
        match self {
            NoteContentModel::Plain(c) => c,
            NoteContentModel::Md(c) => c,
            NoteContentModel::Html(c) => c,
            NoteContentModel::Latex(c) => c,
        }
    }

    pub fn mime_type(&self) -> &'static str {
        match self {
            NoteContentModel::Plain(_) => "text/plain",
            NoteContentModel::Md(_) => "text/markdown",
            NoteContentModel::Html(_) => "text/html",
            NoteContentModel::Latex(_) => "application/x-latex",
        }
    }

    pub fn content_type(&self) -> ContentType {
        use NoteContentModel::*;
        match self {
            Plain(_) => ContentType::Plain,
            Md(_) => ContentType::Md,
            Html(_) => ContentType::Html,
            Latex(_) => ContentType::Latex,
        }
    }
}

async fn get_notes_for_viewer(
    conn: &MaybeTxConn,
    rconn: &KVObject,
    viewer_id: Option<UserID>,
    ids: &[NoteID],
) -> ServiceResult<Vec<DetailedNoteModel>> {
    let mut result = Vec::with_capacity(ids.len());
    for id in ids {
        let note = get_note_by_id_visibility_check(conn, rconn, *id, viewer_id, false).await?;
        match note {
            Some(note) => result.push(note),
            None => {}
        }
    }

    Ok(result)
}

pub async fn get_timeline_notes(
    conn: &MaybeTxConn,
    rconn: &KVObject,
    viewer_id: Option<UserID>,
    include_public: bool,
    limit: u64,
    before_date: Option<DateTime<Utc>>,
) -> ServiceResult<Vec<DetailedNoteModel>> {
    let ids = get_timeline_note_ids(conn, viewer_id, include_public, limit, before_date).await?;
    get_notes_for_viewer(conn, rconn, viewer_id, &ids).await
}

pub async fn get_note_replies(
    conn: &MaybeTxConn,
    rconn: &KVObject,
    viewer_id: Option<UserID>,
    target_note_id: NoteID,
    limit: u64,
    before_date: Option<DateTime<Utc>>,
) -> ServiceResult<Vec<DetailedNoteModel>> {
    let ids = get_note_reply_ids(conn, viewer_id, target_note_id, limit, before_date).await?;
    get_notes_for_viewer(conn, rconn, viewer_id, &ids).await
}

pub async fn get_user_notes(
    conn: &MaybeTxConn,
    rconn: &KVObject,
    viewer_id: Option<UserID>,
    user_id: UserID,
    limit: u64,
    before_date: Option<DateTime<Utc>>,
) -> ServiceResult<Vec<DetailedNoteModel>> {
    let ids = get_user_note_ids(conn, viewer_id, user_id, limit, before_date, true).await?;

    let mut result = Vec::with_capacity(ids.len());
    for id in ids {
        let note = get_note_by_id_visibility_check(conn, rconn, id, viewer_id, false).await?;
        match note {
            Some(note) => result.push(note),
            None => {}
        }
    }

    Ok(result)
}

pub async fn get_user_apub_outbox(
    conn: &MaybeTxConn,
    viewer_id: Option<UserID>,
    user_id: UserID,
    limit: u64,
    before_date: Option<DateTime<Utc>>,
    base_url: &Url,
    data: &Data<MyFederationData>,
) -> ServiceResult<Vec<OutboxActivity>> {
    let ids = get_user_note_ids(conn, viewer_id, user_id, limit, before_date, false).await?;

    let mut notes = Vec::with_capacity(ids.len());
    for id in ids {
        let note =
            get_apubnote_by_id_visibility_check(conn, id, viewer_id, base_url, false).await?;
        match note {
            Some(note) => notes.push(note),
            None => {}
        }
    }

    // convert to create activities
    let mut result = vec![];
    for note in notes {
        result.push(OutboxActivity::from_note_apub(note, conn, data).await?);
    }

    Ok(result)
}

async fn get_user_note_ids(
    conn: &MaybeTxConn,
    viewer_id: Option<UserID>,
    user_id: UserID,
    limit: u64,
    before_date: Option<DateTime<Utc>>,
    include_renotes: bool,
) -> ServiceResult<Vec<NoteID>> {
    let query = Query::select()
        .expr_as(Expr::cust("n.id"), Alias::new("note_id"))
        .from_as(entity::note::Entity, Alias::new("n"))
        .cond_where(
            Condition::all()
                .add(
                    Condition::all()
                        .add(Expr::cust("n.author_id").eq(user_id.as_db()))
                        .add_option(
                            (!include_renotes).then(|| Expr::cust("n.content").is_not_null()),
                        )
                        .add(
                            Condition::any()
                                .add_option(
                                    viewer_id.map(|v| Expr::cust("n.author_id").eq(v.as_db())),
                                ) // self notes
                                .add(Expr::cust("n.visibility").is_in([
                                    ActiveEnum::as_enum(&Visibility::Public),
                                    ActiveEnum::as_enum(&Visibility::Unlisted),
                                ])) // public or unlisted notes
                                .add_option(viewer_id.map(|v| {
                                    Expr::exists(
                                        Query::select()
                                            .column(entity::user_follow::Column::Id)
                                            .from(entity::user_follow::Entity)
                                            .cond_where(
                                                Condition::all()
                                                    .add(
                                                        entity::user_follow::Column::FollowerId
                                                            .eq(v.as_db()),
                                                    )
                                                    .add(
                                                        Expr::col(
                                                            entity::user_follow::Column::FollowedId,
                                                        )
                                                        .eq(Expr::cust("n.author_id")),
                                                    )
                                                    .add(
                                                        entity::user_follow::Column::Pending
                                                            .eq(false),
                                                    ),
                                            )
                                            .take(),
                                    )
                                    .and(
                                        Expr::cust("n.visibility")
                                            .eq(ActiveEnum::as_enum(&Visibility::Follower)),
                                    )
                                })) // follower notes
                                .add_option(viewer_id.map(|v| {
                                    Expr::exists(
                                        Query::select()
                                            .expr_as(
                                                Expr::col(entity::note_mention::Column::NoteId),
                                                Alias::new("note_id"),
                                            )
                                            .from(entity::note_mention::Entity)
                                            .cond_where(
                                                Condition::all()
                                                    .add(Expr::cust("n.id").eq(Expr::col(
                                                        entity::note_mention::Column::NoteId,
                                                    )))
                                                    .add(
                                                        entity::note_mention::Column::TargetUserId
                                                            .eq(v.as_db()),
                                                    ),
                                            )
                                            .to_owned(),
                                    )
                                })), // mentioned notes,
                        ),
                )
                .add_option(before_date.map(|d| Expr::cust("n.created_at").lte(d)))
                .add(Expr::cust("n.deleted_at").is_null()),
        )
        .order_by_expr(Expr::cust("n.created_at"), Order::Desc)
        .limit(limit)
        .to_owned();

    let ids = conn
        .query_all(StatementBuilder::build(
            &query,
            &conn.get_database_backend(),
        ))
        .await
        .map_err_unknown()?;

    Ok(ids
        .into_iter()
        .map(|n| NoteID::from_db_trusted(n.try_get("", "note_id").unwrap()))
        .collect())
}

pub async fn get_user_note_count(
    conn: &MaybeTxConn,
    viewer_id: Option<UserID>,
    user_id: UserID,
    include_renotes: bool,
) -> ServiceResult<i64> {
    let query = Query::select()
        .expr_as(
            Expr::expr(Expr::cust("n.id")).count(),
            Alias::new("note_count"),
        )
        .from_as(entity::note::Entity, Alias::new("n"))
        .cond_where(
            Condition::all()
                .add(
                    Condition::all()
                        .add(Expr::cust("n.author_id").eq(user_id.as_db()))
                        .add_option(
                            (!include_renotes).then(|| Expr::cust("n.content").is_not_null()),
                        )
                        .add(
                            Condition::any()
                                .add_option(
                                    viewer_id.map(|v| Expr::cust("n.author_id").eq(v.as_db())),
                                ) // self notes
                                .add(Expr::cust("n.visibility").is_in([
                                    ActiveEnum::as_enum(&Visibility::Public),
                                    ActiveEnum::as_enum(&Visibility::Unlisted),
                                ])) // public or unlisted notes
                                .add_option(viewer_id.map(|v| {
                                    Expr::exists(
                                        Query::select()
                                            .column(entity::user_follow::Column::Id)
                                            .from(entity::user_follow::Entity)
                                            .cond_where(
                                                Condition::all()
                                                    .add(
                                                        entity::user_follow::Column::FollowerId
                                                            .eq(v.as_db()),
                                                    )
                                                    .add(
                                                        Expr::col(
                                                            entity::user_follow::Column::FollowedId,
                                                        )
                                                        .eq(Expr::cust("n.author_id")),
                                                    )
                                                    .add(
                                                        entity::user_follow::Column::Pending
                                                            .eq(false),
                                                    ),
                                            )
                                            .take(),
                                    )
                                    .and(
                                        Expr::cust("n.visibility")
                                            .eq(ActiveEnum::as_enum(&Visibility::Follower)),
                                    )
                                })) // follower notes
                                .add_option(viewer_id.map(|v| {
                                    Expr::exists(
                                        Query::select()
                                            .expr_as(
                                                Expr::col(entity::note_mention::Column::NoteId),
                                                Alias::new("note_id"),
                                            )
                                            .from(entity::note_mention::Entity)
                                            .cond_where(
                                                Condition::all()
                                                    .add(Expr::cust("n.id").eq(Expr::col(
                                                        entity::note_mention::Column::NoteId,
                                                    )))
                                                    .add(
                                                        entity::note_mention::Column::TargetUserId
                                                            .eq(v.as_db()),
                                                    ),
                                            )
                                            .to_owned(),
                                    )
                                })), // mentioned notes,
                        ),
                )
                .add(Expr::cust("n.deleted_at").is_null()),
        )
        .to_owned();

    let result = conn
        .query_one(StatementBuilder::build(
            &query,
            &conn.get_database_backend(),
        ))
        .await
        .map_err_unknown()?
        .expect("count should be available");

    let count = result.try_get("", "note_count").unwrap();
    Ok(count)
}

#[derive(Debug, Clone, Error, ExpectedError)]
pub enum NoteLikeError {
    #[error("user not found")]
    #[ee(status(StatusCode::NOT_FOUND))]
    UserNotFound,
    #[error("note not found")]
    #[ee(status(StatusCode::NOT_FOUND))]
    NoteNotFound,
}

async fn create_user_list<T: Clone>(
    conn: &MaybeTxConn,
    rconn: &KVObject,
    user_ids: &[(UserID, T)],
) -> ServiceResult<Vec<(SimpleUserModel, T)>> {
    let mut users = vec![];
    for (user_id, data) in user_ids {
        let user = get_user_by_id(conn, rconn, *user_id).await?;
        match user {
            Some(u) => users.push((u, data.clone())),
            None => return Err(ServiceError::ise("user not found while creating user list")),
        }
    }

    Ok(users)
}

pub async fn get_renoted_users(
    conn: &MaybeTxConn,
    rconn: &KVObject,
    viewer_id: Option<UserID>,
    target_note_id: NoteID,
    limit: u64,
    before_date: Option<DateTime<Utc>>,
) -> ServiceResult<Vec<(SimpleUserModel, DateTime<Utc>)>> {
    // visibility check
    let viz = note_visibility_check(conn, target_note_id, viewer_id, false).await?;
    if !viz {
        return create_error_simple(StatusCode::NOT_FOUND, "note not found");
    }

    let renotes = entity::note::Entity::find()
        .find_also_related(entity::user::Entity)
        .filter(
            Condition::all()
                .add(entity::note::Column::RenoteOfId.eq(target_note_id.as_db()))
                .add(entity::note::Column::DeletedAt.is_null()) // この条件はなくてもいいはず
                .add_option(before_date.map(|d| entity::note::Column::CreatedAt.lte(d))),
            // renote はすべて public or unlisted なので visibility は見なくてよい
        )
        .order_by_desc(entity::note::Column::CreatedAt)
        .limit(limit)
        .all(conn)
        .await
        .map_err_unknown()?;

    let user_ids: Vec<_> = renotes
        .into_iter()
        .map(|(r, n)| {
            (
                UserID::from_db_trusted(n.unwrap().id),
                r.created_at.clone().and_utc(),
            )
        })
        .collect();

    create_user_list(conn, rconn, &user_ids).await
}

pub async fn get_liked_users(
    conn: &MaybeTxConn,
    rconn: &KVObject,
    viewer_id: Option<UserID>,
    target_note_id: NoteID,
    limit: u64,
    before_date: Option<DateTime<Utc>>,
) -> ServiceResult<Vec<(SimpleUserModel, DateTime<Utc>)>> {
    // visibility check
    let viz = note_visibility_check(conn, target_note_id, viewer_id, false).await?;
    if !viz {
        return create_error_simple(StatusCode::NOT_FOUND, "note not found");
    }

    let likes = entity::note_like::Entity::find()
        .find_also_related(entity::user::Entity)
        .filter(
            Condition::all()
                .add(entity::note_like::Column::NoteId.eq(target_note_id.as_db()))
                .add(entity::note_like::Column::IsPrivate.eq(false))
                .add_option(before_date.map(|d| entity::note_like::Column::CreatedAt.lte(d))),
        )
        .order_by_desc(entity::note_like::Column::CreatedAt)
        .limit(limit)
        .all(conn)
        .await
        .map_err_unknown()?;

    let user_ids: Vec<_> = likes
        .into_iter()
        .map(|(like, n)| {
            (
                UserID::from_db_trusted(n.unwrap().id),
                like.created_at.clone().and_utc(),
            )
        })
        .collect();

    create_user_list(conn, rconn, &user_ids).await
}
