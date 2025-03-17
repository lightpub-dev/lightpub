use std::path::PathBuf;

use activitypub_federation::{config::Data, traits::Object};
use actix_web::http::StatusCode;
use chrono::{DateTime, Utc};
use derive_builder::Builder;
use expected_error_derive::ExpectedError;
use itertools::Itertools;
use sea_orm::{
    AccessMode, ActiveModelTrait, ColumnTrait, Condition, EntityTrait, IntoActiveModel,
    IsolationLevel, QueryFilter, Set, TryIntoModel,
};
use thiserror::Error;
use tracing::{debug, warn};
use url::Url;

use crate::{
    MyFederationData,
    services::{
        MapToUnknown, ServiceError, ServiceResult, UpsertOperation,
        apub::{CreateActivity, UpdateActivity},
        create_error_simple,
        db::{Conn, MaybeTxConn},
        id::{Identifier, NoteID, UploadID, UserID},
        kv::KVObject,
        notification::{NotificationBody, add_notification},
        queue::QConn,
        user::{
            UserSpecifier, get_apubuser_by_id, get_user_by_id, get_user_by_spec_with_remote,
            is_blocking_or_blocked,
        },
    },
    utils::sanitize::CleanString,
};

use super::{
    ContentType, Mention, VisibilityModel, get::get_apubnote_by_id_with_inboxes, get_note_by_id,
    hashtag::find_hashtags, invalidate_note_basic_cache, mention::find_mentions,
    note_visibility_check,
};

pub async fn upsert_note(
    conn: &Conn,
    rconn: &KVObject,
    qconn: &QConn,
    updated_note_id: Option<ExistingNote>,
    author_id: UserID,
    content: &str,
    content_type: ContentType,
    visibility: Option<VisibilityModel>, // should be set when inserting
    options: &PostCreateOptions,
    my_domain: &str,
    base_url: &Url,
    fed_data: &Data<MyFederationData>,
) -> ServiceResult<NoteID> {
    let tx = conn
        .as_tx_with_config(IsolationLevel::ReadCommitted, AccessMode::ReadWrite)
        .await?
        .into();

    validate_note_content(content)?;

    // find author
    let author = get_user_by_spec_with_remote(
        &tx,
        rconn,
        &UserSpecifier::ID(author_id.clone()),
        my_domain,
        fed_data,
    )
    .await?;
    let author = match author {
        None => return Err(ServiceError::known(NoteCreateError::AuthorNotFound)),
        Some(a) => a,
    };

    // Get current model
    let (note_id, mut model, is_update, current_reply_to_id, current_visibility) =
        match updated_note_id {
            None => {
                let note_id = NoteID::new_random();
                let model = entity::note::ActiveModel {
                    ..Default::default()
                };
                (note_id, model, false, None, None)
            }
            Some(ExistingNote::ByID(id)) => {
                let model = entity::note::Entity::find()
                    .filter(
                        Condition::all()
                            .add(entity::note::Column::Id.eq(id.as_db()))
                            .add(entity::note::Column::DeletedAt.is_null()),
                    )
                    .one(&tx)
                    .await
                    .map_err_unknown()?;
                match model {
                    None => return Err(ServiceError::known(NoteCreateError::UpdateNoteNotFound)),
                    Some(m) => {
                        let reply_to_id = m.reply_to_id.clone().map(NoteID::from_db_trusted);
                        let visibility = VisibilityModel::from_db(m.visibility.clone());
                        (
                            id,
                            m.into_active_model(),
                            true,
                            reply_to_id,
                            Some(visibility),
                        )
                    }
                }
            }
            Some(ExistingNote::ByURL(ref url)) => {
                let model = entity::note::Entity::find()
                    .filter(
                        Condition::all()
                            .add(entity::note::Column::Url.eq(url.as_str()))
                            .add(entity::note::Column::DeletedAt.is_null()),
                    )
                    .one(&tx)
                    .await
                    .map_err_unknown()?;
                match model {
                    None => {
                        let note_id = NoteID::new_random();
                        let model = entity::note::ActiveModel {
                            ..Default::default()
                        };
                        (note_id, model.into_active_model(), false, None, None)
                    }
                    Some(m) => {
                        let reply_to_id = m.reply_to_id.clone().map(NoteID::from_db_trusted);
                        let visibility = VisibilityModel::from_db(m.visibility.clone());
                        (
                            NoteID::from_db_trusted(m.id.clone()),
                            m.into_active_model(),
                            true,
                            reply_to_id,
                            Some(visibility),
                        )
                    }
                }
            }
        };
    let url = match updated_note_id {
        Some(ExistingNote::ByURL(url)) => Some(url),
        _ => None,
    };

    let visibility = match (visibility, current_visibility) {
        (Some(v), None) => v,
        (None, Some(v)) => v,
        (None, None) => return Err(ServiceError::ise("visibility not set for new note")),
        (Some(_), Some(_)) => {
            return Err(ServiceError::known(NoteCreateError::VisibilitySetOnUpdate));
        }
    };

    // Get reply to id
    let (reply_to_id, replied_note) = match &options.reply_to_id {
        UpsertOperation::Set(Some(r)) => {
            let rn = get_note_by_id(&tx, *r, false).await?;
            match rn {
                Some(rn) if note_visibility_check(&tx, rn.id, Some(author_id), false).await? => {
                    // visibility compatibility check
                    if !rn.visibility.reply_compatible(visibility) {
                        return Err(ServiceError::known(NoteCreateError::InvalidReplyVisibility));
                    }

                    (UpsertOperation::Set(Some(rn.id)), Some(rn))
                }
                _ => return Err(ServiceError::known(NoteCreateError::RepliedNoteNotFound)),
            }
        }
        UpsertOperation::Set(None) => (UpsertOperation::Set(None), None),
        UpsertOperation::KeepOrSetDefault => {
            match current_reply_to_id {
                None => (UpsertOperation::KeepOrSetDefault, None),
                Some(rep) => {
                    let replied_note = get_note_by_id(&tx, rep, false).await?;
                    match replied_note {
                        None => {
                            // replied note is deleted
                            (UpsertOperation::KeepOrSetDefault, None)
                        }
                        Some(rn) => (UpsertOperation::KeepOrSetDefault, Some(rn)),
                    }
                }
            }
        }
    };

    let hashtags = note_create_find_hashtags(content, content_type, &options.hashtags_override);
    let mentions: Vec<UserID> = note_create_find_mentions(
        &tx,
        rconn,
        content,
        content_type,
        &options.mentions_override,
        replied_note.as_ref().map(|n| n.author.id),
        my_domain,
        fed_data,
    )
    .await?;

    let content = match content_type {
        ContentType::Html => CleanString::clean(content).into_inner(),
        _ => content.to_string(),
    };

    let now_time = options.created_at.unwrap_or_else(Utc::now);
    // base note
    set_note_active_model(
        &mut model,
        note_id,
        author_id,
        content,
        content_type,
        visibility,
        reply_to_id,
        UpsertOperation::Set(url.clone()),
        UpsertOperation::Set(options.view_url.clone()),
        options.sensitive,
        is_update,
        now_time,
    );
    if is_update {
        model.update(&tx).await.map_err_unknown()?;
    } else {
        model.insert(&tx).await.map_err_unknown()?;
    }

    // hashtags
    // delete all tags first
    entity::note_tag::Entity::delete_many()
        .filter(entity::note_tag::Column::NoteId.eq(note_id.as_db()))
        .exec(&tx)
        .await
        .map_err_unknown()?;
    // then insert new tags
    for tag_name in hashtags {
        let tag_id = create_or_get_tag_id(&tx, &tag_name).await?;
        let note_tag = entity::note_tag::ActiveModel {
            note_id: Set(note_id.as_db()),
            tag_id: Set(tag_id.id),
            ..Default::default()
        };
        note_tag.insert(&tx).await.map_err_unknown()?;
    }

    // reply notification
    if let Some(replied_note) = &replied_note {
        let body = NotificationBody::Replied(author_id, note_id, replied_note.id);
        add_notification(&tx, replied_note.author.id, &body).await?;
    }

    // mentions
    // delete all mentions first
    entity::note_mention::Entity::delete_many()
        .filter(entity::note_mention::Column::NoteId.eq(note_id.as_db()))
        .exec(&tx)
        .await
        .map_err_unknown()?;
    // then insert new mentions
    for mention in mentions {
        // ブロックしている or されている場合は対象から外す
        let block = is_blocking_or_blocked(&tx, author_id, mention).await?;
        if block {
            continue;
        }

        let mention_model = entity::note_mention::ActiveModel {
            note_id: Set(note_id.as_db()),
            target_user_id: Set(mention.as_db()),
            ..Default::default()
        };
        mention_model.insert(&tx).await.map_err_unknown()?;

        // if already notified, skip notification
        if replied_note
            .as_ref()
            .is_some_and(|r| r.author.id == mention)
        {
            continue;
        }
        add_mention_notification_if_local(&tx, rconn, author_id, note_id, mention).await?;
    }

    // uploads
    if let UpsertOperation::Set(uploads) = &options.uploads {
        // delete all uploads first
        entity::note_upload::Entity::delete_many()
            .filter(entity::note_upload::Column::NoteId.eq(note_id.as_db()))
            .exec(&tx)
            .await
            .map_err_unknown()?;
        // then insert new uploads
        for upload in uploads {
            let upload_id = match upload {
                NoteUpload::File(upload_id, filename, mime_type) => {
                    let model = entity::upload::ActiveModel {
                        id: Set(upload_id.as_db()),
                        filename: Set(Some(filename.to_str().expect("bad filename").to_string())),
                        mime_type: Set(mime_type.to_string()),
                        ..Default::default()
                    };
                    model.insert(&tx).await.map_err_unknown()?;
                    upload_id.clone()
                }
                NoteUpload::URL(upload_id, url, mime_type) => {
                    let model = entity::upload::ActiveModel {
                        id: Set(upload_id.as_db()),
                        url: Set(Some(url.to_string())),
                        mime_type: Set(mime_type.to_string()),
                        ..Default::default()
                    };
                    model.insert(&tx).await.map_err_unknown()?;
                    upload_id.clone()
                }
            };
            let model = entity::note_upload::ActiveModel {
                note_id: Set(note_id.as_db()),
                upload_id: Set(upload_id.as_db()),
                ..Default::default()
            };
            model.insert(&tx).await.map_err_unknown()?;
        }
    }

    tx.commit().await?;

    invalidate_note_basic_cache(rconn, note_id).await?;

    // send activitypub note
    let tx = conn.as_tx().await?.into();
    if author.is_local() {
        let (note, inboxes) = get_apubnote_by_id_with_inboxes(&tx, note_id, None, base_url, false)
            .await?
            .expect("upserted note not found");
        let note_obj = note.into_json(fed_data).await?;
        let author_obj = get_apubuser_by_id(&tx, author_id, base_url)
            .await?
            .expect("author should exist");
        if is_update {
            // Update activity
            let activity = UpdateActivity::from_note(note_obj);
            debug!("sending note to inbox: {inboxes:?}");
            debug!("apub note update: {activity:?}");
            qconn.queue_activity(activity, author_obj, inboxes).await?;
        } else {
            // Create acitvity
            let activity = CreateActivity::from_note(note_obj);
            debug!("sending note to inbox: {inboxes:?}");
            debug!("apub note create: {activity:?}");
            qconn.queue_activity(activity, author_obj, inboxes).await?;
        }
    }

    Ok(note_id)
}

async fn create_or_get_tag_id(
    tx: &MaybeTxConn,
    tag_name: &str,
) -> ServiceResult<entity::tag::Model> {
    // check for existing tag
    let tag = entity::tag::Entity::find()
        .filter(entity::tag::Column::Name.eq(tag_name))
        .one(tx)
        .await
        .map_err_unknown()?;

    match tag {
        Some(tag) => Ok(tag),
        None => {
            // create a new tag
            let new_tag = entity::tag::ActiveModel {
                name: Set(tag_name.to_owned()),
                ..Default::default()
            };
            let new_tag = new_tag.save(tx).await.map_err_unknown()?;

            Ok(new_tag.try_into_model().map_err_unknown()?)
        }
    }
}

fn set_note_active_model(
    model: &mut entity::note::ActiveModel,
    note_id: NoteID,
    author_id: UserID,
    content: String,
    content_type: ContentType,
    visibility: VisibilityModel,
    reply_to_id: UpsertOperation<Option<NoteID>>,
    url: UpsertOperation<Option<Url>>,
    view_url: UpsertOperation<Option<Url>>,
    sensitive: UpsertOperation<bool>,
    is_update: bool,
    created_or_updated_at: DateTime<Utc>,
) {
    model.id = Set(note_id.as_db());
    model.author_id = Set(author_id.as_db());
    model.content = Set(Some(content));
    model.content_type = Set(Some(content_type.as_str().to_string()));
    model.visibility = Set(visibility.as_db());
    if let UpsertOperation::Set(rep) = reply_to_id {
        model.reply_to_id = Set(rep.map(|r| r.as_db()));
    }
    if let UpsertOperation::Set(url) = url {
        model.url = Set(url.map(|s| s.to_string()));
    }
    if let UpsertOperation::Set(view_url) = view_url {
        model.view_url = Set(view_url.map(|s| s.to_string()));
    }
    if let UpsertOperation::Set(s) = sensitive {
        model.sensitive = Set(s as i8);
    }

    if is_update {
        model.updated_at = Set(Some(created_or_updated_at.naive_utc()));
    } else {
        model.created_at = Set(created_or_updated_at.naive_utc());
    }
}

async fn add_mention_notification_if_local(
    tx: &MaybeTxConn,
    rconn: &KVObject,
    author_id: UserID,
    note_id: NoteID,
    target_user_id: UserID,
) -> ServiceResult<()> {
    let target_user = get_user_by_id(tx, rconn, target_user_id).await?;
    if let Some(t) = target_user {
        if t.is_local() {
            let body = NotificationBody::Mentioned(author_id.clone(), note_id.clone());
            add_notification(tx, target_user_id, &body).await?;
        }
    }

    Ok(())
}

fn note_create_find_hashtags(
    content: &str,
    content_type: ContentType,
    hashtag_override: &Option<Vec<String>>,
) -> Vec<String> {
    if let Some(hashtag_override) = hashtag_override {
        hashtag_override.clone()
    } else if content_type != ContentType::Plain {
        vec![] // TODO: parse hashtags from content
    } else {
        find_hashtags(content)
            .into_iter()
            .map(|h| h.hashtag)
            .collect()
    }
    .into_iter()
    .unique()
    .collect()
}

async fn note_create_find_mentions(
    conn: &MaybeTxConn,
    rconn: &KVObject,
    content: &str,
    content_type: ContentType,
    mention_override: &Option<Vec<Mention>>,
    replied_note_author: Option<UserID>,
    my_domain: &str,
    data: &Data<MyFederationData>,
) -> ServiceResult<Vec<UserID>> {
    let mut mentions = if let Some(mention_override) = &mention_override {
        let mut ids = vec![];
        for mention in mention_override {
            let user_spec = mention.clone().into_user_spec();
            let user =
                get_user_by_spec_with_remote(conn, rconn, &user_spec, my_domain, data).await?;
            match user {
                Some(u) => ids.push(u.id().clone()),
                None => {}
            }
        }
        ids
    } else if content_type != ContentType::Plain {
        vec![] // TODO: parse mentions from content
    } else {
        let mentions = find_mentions(content, my_domain);
        let mut ids = vec![];
        for mention in &mentions {
            let user = get_user_by_spec_with_remote(
                conn,
                rconn,
                &UserSpecifier::Username(mention.username().clone(), mention.domain().clone()),
                my_domain,
                data,
            )
            .await?;
            match user {
                Some(u) => ids.push(u.id().clone()),
                None => {
                    warn!("mentioned user not found: {:?}", mention);
                }
            }
        }
        ids
    };

    if let Some(replied_note_author) = replied_note_author {
        mentions.push(replied_note_author.clone());
    }

    Ok(mentions.into_iter().unique().collect())
}

#[derive(Debug, Clone)]
pub enum ExistingNote {
    ByID(NoteID),
    ByURL(Url),
}

#[derive(Debug, Clone)]
pub enum NoteUpload {
    File(UploadID, PathBuf, String), // filename, mime_type
    URL(UploadID, Url, String),      // url, mime_type
}

/// ノート作成または更新時の追加オプション。
/// [PostCreateOptionsBuilder] を使用して構築すること。
#[derive(Debug, Clone, Builder)]
pub struct PostCreateOptions {
    /// 返信先ノートの ID。デフォルトは None。
    #[builder(default = "UpsertOperation::KeepOrSetDefault")]
    reply_to_id: UpsertOperation<Option<NoteID>>,
    /// 添付ファイルのリスト。デフォルトは空。
    #[builder(default = "UpsertOperation::KeepOrSetDefault")]
    uploads: UpsertOperation<Vec<NoteUpload>>,
    /// ハッシュタグのリスト。Some の場合は本文中からハッシュタグを検索せず、このリストの値が採用される。
    #[builder(default)]
    hashtags_override: Option<Vec<String>>,
    /// メンションのリスト。Some の場合は本文中からメンションを検索せず、このリストの値が採用される。
    #[builder(default)]
    mentions_override: Option<Vec<Mention>>,
    /// ブラウザで表示する際の URL。リモートノートの場合のみ必要。
    #[builder(default)]
    view_url: Option<Url>,
    /// ノート作成日時。None の場合は現在時刻が使用される。
    #[builder(default)]
    created_at: Option<DateTime<Utc>>,
    /// ノートをセンシティブ設定するか否か。デフォルトは false。
    #[builder(default = "UpsertOperation::KeepOrSetDefault")]
    sensitive: UpsertOperation<bool>,
}

impl Default for PostCreateOptions {
    fn default() -> Self {
        Self {
            reply_to_id: UpsertOperation::KeepOrSetDefault,
            uploads: UpsertOperation::KeepOrSetDefault,
            mentions_override: None,
            hashtags_override: None,
            view_url: None,
            created_at: None,
            sensitive: UpsertOperation::KeepOrSetDefault,
        }
    }
}

#[derive(Debug, Clone, Error, ExpectedError)]
pub enum NoteCreateError {
    #[error("Update note not found")]
    #[ee(status(StatusCode::NOT_FOUND))]
    UpdateNoteNotFound,
    #[error("Author not found")]
    #[ee(status(StatusCode::NOT_FOUND))]
    AuthorNotFound,
    #[error("Replied note not found")]
    #[ee(status(StatusCode::NOT_FOUND))]
    RepliedNoteNotFound,
    #[error("renote target not found")]
    #[ee(status(StatusCode::NOT_FOUND))]
    RenoteTargetNotFound,
    #[error("renote not allowed for this note")]
    #[ee(status(StatusCode::FORBIDDEN))]
    RenoteNotAllowed,
    #[error("only public or unlisted are allowed for renote")]
    #[ee(status(StatusCode::BAD_REQUEST))]
    InvalidRenoteVisibility,
    #[error("you cannot reply to this note with current visibility")]
    #[ee(status(StatusCode::BAD_REQUEST))]
    InvalidReplyVisibility,
    #[error("you can only renote once")]
    #[ee(status(StatusCode::BAD_REQUEST))]
    AlreadyRenoted,
    #[error("you cannot change visibility on update")]
    #[ee(status(StatusCode::BAD_REQUEST))]
    VisibilitySetOnUpdate,
    // #[error("attachment is invalid")]
    // InvalidAttachment,
}

pub fn validate_note_content(content: &str) -> ServiceResult<()> {
    if content.len() > 50000 {
        return create_error_simple(StatusCode::BAD_REQUEST, "note content too long");
    }
    Ok(())
}
