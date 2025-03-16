use crate::{
    ServiceResult,
    services::{
        MapToUnknown,
        db::MaybeTxConn,
        id::{Identifier, NoteID, UserID},
        kv::KVObject,
        user::{domain_to_optional, get_apubuser_by_id, get_user_by_id},
    },
};
use sea_orm::{Condition, prelude::*};
use url::Url;

use super::{
    ApubDataWithInboxes, BasicNoteModel, CalculateToAndCcResult, ContentType, DetailedNoteModel,
    NoteApubData, NoteApubHashtagData, NoteApubMentionData, NoteAuthorModel, NoteContentModel,
    NoteMentionModel, NoteModelDetails, NoteSpecifier, NoteWithApubModel, VisibilityModel,
    calculate_to_and_cc, note_visibility_check,
    upload::{get_note_uploads, get_note_uploads_apub},
};

/// ノートを取得する。
/// 可視性チェックは行わない。
pub async fn get_note_by_spec(
    conn: &MaybeTxConn,
    spec: &NoteSpecifier,
    my_domain: &str,
    include_deleted: bool,
) -> ServiceResult<Option<BasicNoteModel>> {
    let note = get_rawnote_by_spec(conn, spec, my_domain).await?;
    let note = match note {
        None => return Ok(None),
        Some((note, author)) => (note, author),
    };
    get_note_by_spec_impl(conn, note, include_deleted).await
}

/// ノートを取得する。
/// 可視性チェックは行わない。
pub async fn get_note_by_id(
    conn: &MaybeTxConn,
    note_id: NoteID,
    include_deleted: bool,
) -> ServiceResult<Option<BasicNoteModel>> {
    let note = get_rawnote_by_id(conn, note_id).await?;
    let note = match note {
        None => return Ok(None),
        Some((note, author)) => (note, author),
    };
    get_note_by_spec_impl(conn, note, include_deleted).await
}

/// ノートを取得する。
/// 可視性チェックも行い、失敗した場合は None を返す。
pub async fn get_note_by_id_visibility_check(
    conn: &MaybeTxConn,
    rconn: &KVObject,
    note_id: NoteID,
    viewer_id: Option<UserID>,
    include_deleted: bool,
) -> ServiceResult<Option<DetailedNoteModel>> {
    let note =
        get_note_by_id_with_viewer_specific_info(conn, rconn, note_id, viewer_id, include_deleted)
            .await?;
    match note {
        None => Ok(None),
        Some(note) => {
            if note_visibility_check(conn, note_id, viewer_id, include_deleted).await? {
                Ok(Some(note))
            } else {
                Ok(None)
            }
        }
    }
}

/// ノートを取得する。
/// 可視性チェックは行わない。
pub async fn get_apubnote_by_id(
    conn: &MaybeTxConn,
    note_id: NoteID,
    base_url: &Url,
    include_deleted: bool,
) -> ServiceResult<Option<NoteWithApubModel>> {
    get_apubnote_by_id_with_inboxes(conn, note_id, None, base_url, include_deleted)
        .await
        .map(|r| r.map(|(n, _)| n))
}

/// ノートを取得する。
/// 可視性チェックも行い、失敗した場合は None を返す。
pub async fn get_apubnote_by_id_visibility_check(
    conn: &MaybeTxConn,
    note_id: NoteID,
    viewer_id: Option<UserID>,
    base_url: &Url,
    include_deleted: bool,
) -> ServiceResult<Option<NoteWithApubModel>> {
    get_apubnote_by_id_with_inboxes(conn, note_id, Some(viewer_id), base_url, include_deleted)
        .await
        .map(|r| r.map(|(n, _)| n))
}

/// ノートを取得する。
/// 可視性チェックは行わない。
pub async fn get_apubnote_by_spec(
    conn: &MaybeTxConn,
    spec: &NoteSpecifier,
    my_domain: &str,
    base_url: &Url,
) -> ServiceResult<Option<NoteWithApubModel>> {
    get_apubnote_by_spec_with_inboxes(conn, spec, None, my_domain, base_url)
        .await
        .map(|r| r.map(|(n, _)| n))
}

async fn get_note_by_id_with_viewer_specific_info(
    conn: &MaybeTxConn,
    rconn: &KVObject,
    note_id: NoteID,
    viewer_id: Option<UserID>,
    include_deleted: bool,
) -> ServiceResult<Option<DetailedNoteModel>> {
    let note = get_note_by_id(conn, note_id, include_deleted).await?;
    match note {
        Some(note) => fill_note_user_interaction(conn, rconn, note, viewer_id)
            .await
            .map(|n| Some(n)),
        None => Ok(None),
    }
}

async fn get_note_by_spec_impl(
    conn: &MaybeTxConn,
    (note, author): (entity::note::Model, entity::user::Model),
    include_deleted: bool,
) -> ServiceResult<Option<BasicNoteModel>> {
    if !include_deleted && note.deleted_at.is_some() {
        return Ok(None);
    }

    let content = match (note.content, note.content_type) {
        (Some(c), Some(t)) => match ContentType::from_str(&t) {
            None => panic!("invalid content type"),
            Some(ContentType::Plain) => Some(NoteContentModel::Plain(c)),
            Some(ContentType::Md) => Some(NoteContentModel::Md(c)),
            Some(ContentType::Html) => Some(NoteContentModel::Html(c)),
            Some(ContentType::Latex) => Some(NoteContentModel::Latex(c)),
        },
        (None, None) => None,
        _ => panic!("content and content_type must be both set or both unset"),
    };

    let note_id = NoteID::from_db_trusted(note.id);

    let uploads = get_note_uploads(conn, note_id).await?;

    let visibility = VisibilityModel::from_db(note.visibility);
    let note_model = BasicNoteModel {
        id: note_id,
        author: NoteAuthorModel {
            id: UserID::from_db_trusted(author.id),
            username: author.username,
            nickname: author.nickname,
            domain: domain_to_optional(author.domain),
        },
        content,
        visibility,
        created_at: note.created_at.to_utc(),
        updated_at: note.updated_at.map(|d| d.to_utc()),
        reply_to_id: note.reply_to_id.map(NoteID::from_db_trusted),
        renote_of_id: note.renote_of_id.map(NoteID::from_db_trusted),
        deleted_at: note.deleted_at.map(|d| d.to_utc()),
        uploads,
        sensitive: note.sensitive,
    };
    Ok(Some(note_model))
}

async fn get_rawnote_by_id(
    conn: &MaybeTxConn,
    note_id: NoteID,
) -> ServiceResult<Option<(entity::note::Model, entity::user::Model)>> {
    let note = entity::note::Entity::find_by_id(note_id.as_db())
        .find_also_related(entity::user::Entity)
        .one(conn)
        .await
        .map_err_unknown()?;

    let note = note.map(|(n, a)| (n, a.unwrap()));
    Ok(note)
}

async fn get_rawnote_by_spec(
    conn: &MaybeTxConn,
    spec: &NoteSpecifier,
    my_domain: &str,
) -> ServiceResult<Option<(entity::note::Model, entity::user::Model)>> {
    let spec = spec.local_url_convert(my_domain)?;

    let note = match spec.as_ref() {
        NoteSpecifier::ID(id) => get_rawnote_by_id(conn, *id).await?,
        NoteSpecifier::URL(url) => {
            let note = entity::note::Entity::find()
                .filter(entity::note::Column::Url.eq(url.to_string()))
                .find_also_related(entity::user::Entity)
                .one(conn)
                .await
                .map_err_unknown()?;

            note.map(|(n, a)| (n, a.unwrap()))
        }
    };
    Ok(note)
}

async fn get_apubdata_by_note_id_with_inboxes(
    conn: &MaybeTxConn,
    note_id: NoteID,
    base_url: &Url,
) -> ServiceResult<Option<ApubDataWithInboxes>> {
    let note = get_rawnote_by_id(conn, note_id).await?;

    match note {
        None => Ok(None),
        Some((note, _author)) => {
            let url = get_url_of_note_model(&note, base_url);

            let in_reply_to_url = match &note.reply_to_id {
                None => None,
                Some(reply_to_id) => {
                    let reply_to_id = NoteID::from_db_trusted(reply_to_id.clone());
                    let reply_to_note = get_rawnote_by_id(conn, reply_to_id).await?;
                    match reply_to_note {
                        None => None,
                        Some((reply_to_note, _)) => {
                            Some(get_url_of_note_model(&reply_to_note, base_url))
                        }
                    }
                }
            };

            let author_model = get_apubuser_by_id(
                conn,
                UserID::from_db_trusted(note.author_id.clone()),
                base_url,
            )
            .await?
            .expect("author_model");
            let author_url = author_model.apub.url;

            let CalculateToAndCcResult { to, cc, inboxes } = calculate_to_and_cc(
                conn,
                note_id,
                UserID::from_db_trusted(note.author_id),
                VisibilityModel::from_db(note.visibility.clone()),
                false,
                base_url,
            )
            .await?;

            let view_url = get_view_url_of_note_model(&note, base_url);

            let mentions = {
                let mentioned_user_ids = get_note_mentions(conn, note_id).await?;
                let mut result = vec![];
                for user_id in mentioned_user_ids {
                    let user = get_apubuser_by_id(conn, user_id, base_url)
                        .await?
                        .expect("mentioned user should exist");
                    result.push(NoteApubMentionData {
                        url: user.apub.url,
                        name: user.basic.specifier,
                    });
                }
                result
            };

            let hashtags = {
                let hashtag_names = get_note_hashtags(conn, note_id).await?;
                let mut result = vec![];
                for name in hashtag_names {
                    result.push(NoteApubHashtagData {
                        name: name.clone(),
                        url: base_url
                            .join(&format!("/tag/{}", &urlencoding::encode(&name)))
                            .unwrap(),
                    })
                }
                result
            };

            let attachments = get_note_uploads_apub(conn, note_id, base_url).await?;

            Ok(Some(ApubDataWithInboxes {
                data: NoteApubData {
                    url,
                    fetched_at: note.fetched_at.map(|d| d.to_utc()),
                    author_url,
                    to,
                    cc,
                    in_reply_to_url,
                    view_url: Some(view_url),
                    mentions,
                    hashtags,
                    attachments,
                },
                inboxes,
            }))
        }
    }
}

async fn get_note_mentions(tx: &MaybeTxConn, note_id: NoteID) -> ServiceResult<Vec<UserID>> {
    let mentions = entity::note_mention::Entity::find()
        .filter(entity::note_mention::Column::NoteId.eq(note_id.as_db()))
        .all(tx)
        .await
        .map_err_unknown()?;

    let mut result = vec![];
    for mention in mentions {
        result.push(UserID::from_db_trusted(mention.target_user_id));
    }
    Ok(result)
}

async fn get_note_hashtags(tx: &MaybeTxConn, note_id: NoteID) -> ServiceResult<Vec<String>> {
    let tags = entity::note_tag::Entity::find()
        .find_also_related(entity::tag::Entity)
        .filter(entity::note_tag::Column::NoteId.eq(note_id.as_db()))
        .all(tx)
        .await
        .map_err_unknown()?;

    let mut result = vec![];
    for (_, tag) in tags {
        result.push(tag.unwrap().name);
    }
    Ok(result)
}

pub async fn get_apubnote_by_id_with_inboxes(
    conn: &MaybeTxConn,
    note_id: NoteID,
    viewer_check: Option<Option<UserID>>,
    base_url: &Url,
    include_deleted: bool,
) -> ServiceResult<Option<(NoteWithApubModel, Vec<Url>)>> {
    let note = get_note_by_id(conn, note_id, include_deleted).await?;
    let note = match note {
        None => return Ok(None),
        Some(note) => note,
    };

    build_apubnote_with_inboxes_from_note_model(conn, viewer_check, base_url, note).await
}

async fn get_apubnote_by_spec_with_inboxes(
    conn: &MaybeTxConn,
    spec: &NoteSpecifier,
    viewer_check: Option<Option<UserID>>,
    my_domain: &str,
    base_url: &Url,
) -> ServiceResult<Option<(NoteWithApubModel, Vec<Url>)>> {
    let note = get_note_by_spec(conn, spec, my_domain, false).await?;
    let note = match note {
        None => return Ok(None),
        Some(note) => note,
    };

    build_apubnote_with_inboxes_from_note_model(conn, viewer_check, base_url, note).await
}

async fn build_apubnote_with_inboxes_from_note_model(
    conn: &MaybeTxConn,
    viewer_check: Option<Option<UserID>>,
    base_url: &Url,
    note: BasicNoteModel,
) -> Result<Option<(NoteWithApubModel, Vec<Url>)>, crate::services::ServiceError> {
    // visibility check
    if let Some(viewer_id) = viewer_check {
        let ok = note_visibility_check(conn, note.id, viewer_id, false).await?;
        if !ok {
            return Ok(None);
        }
    }

    let ApubDataWithInboxes { data, inboxes } =
        get_apubdata_by_note_id_with_inboxes(conn, note.id, base_url)
            .await?
            .expect("note should exist");

    Ok(Some((
        NoteWithApubModel {
            basic: note,
            apub: data,
        },
        inboxes,
    )))
}

async fn fill_note_user_interaction(
    conn: &MaybeTxConn,
    rconn: &KVObject,
    note: BasicNoteModel,
    viewer_id: Option<UserID>,
) -> ServiceResult<DetailedNoteModel> {
    let (renoted, liked, bookmarked) = if let Some(viewer_id) = viewer_id {
        // renote
        let renote = entity::note::Entity::find()
            .filter(
                Condition::all()
                    .add(entity::note::Column::AuthorId.eq(viewer_id.as_db()))
                    .add(entity::note::Column::RenoteOfId.eq(note.id.as_db()))
                    .add(entity::note::Column::DeletedAt.is_null()), // これはなくてもいいはず
            )
            .one(conn)
            .await
            .map_err_unknown()?;
        let renoted = Some(renote.is_some());

        // like and bookmark
        let likes = entity::note_like::Entity::find()
            .filter(
                Condition::all()
                    .add(entity::note_like::Column::NoteId.eq(note.id.as_db()))
                    .add(entity::note_like::Column::UserId.eq(viewer_id.as_db())),
            )
            .all(conn)
            .await
            .map_err_unknown()?;
        let mut liked = false;
        let mut bookmarked = false;
        for like in likes {
            if like.is_private {
                bookmarked = true;
            } else {
                liked = true;
            }
        }
        let liked = Some(liked);
        let bookmarked = Some(bookmarked);
        (renoted, liked, bookmarked)
    } else {
        (None, None, None)
    };

    // counters
    let reply_count = entity::note::Entity::find()
        .filter(
            Condition::all()
                .add(entity::note::Column::ReplyToId.eq(note.id.as_db()))
                .add(entity::note::Column::DeletedAt.is_null()), // これはなくてもいいはず
        )
        .count(conn)
        .await
        .map_err_unknown()?;

    let renote_count = entity::note::Entity::find()
        .filter(
            Condition::all()
                .add(entity::note::Column::RenoteOfId.eq(note.id.as_db()))
                .add(entity::note::Column::DeletedAt.is_null()), // これはなくてもいいはず
        )
        .count(conn)
        .await
        .map_err_unknown()?;

    let like_count = entity::note_like::Entity::find()
        .filter(
            Condition::all()
                .add(entity::note_like::Column::NoteId.eq(note.id.as_db()))
                .add(entity::note_like::Column::IsPrivate.eq(false)),
        )
        .count(conn)
        .await
        .map_err_unknown()?;

    // hashtags
    let hashtags = get_note_hashtags(conn, note.id).await?;

    // mentions
    let mention_ids = get_note_mentions(conn, note.id).await?;
    let mut mentions = vec![];
    for mention_id in mention_ids {
        let user = get_user_by_id(conn, rconn, mention_id)
            .await?
            .expect("mentioned user should exist");
        mentions.push(NoteMentionModel {
            id: user.id,
            username: user.username,
            nickname: user.nickname,
            domain: user.domain,
        });
    }

    Ok(DetailedNoteModel {
        basic: note,
        details: NoteModelDetails {
            reply_count,
            renote_count,
            like_count,
            renoted,
            liked,
            bookmarked,
            hashtags,
            mentions,
        },
    })
}

pub fn get_url_of_note_model(note: &entity::note::Model, base_url: &Url) -> Url {
    if let Some(url) = &note.url {
        Url::parse(&url).expect("url should be valid")
    } else {
        base_url
            .join(&format!(
                "/note/{}",
                NoteID::from_db_trusted(note.id.clone())
            ))
            .unwrap()
    }
}

fn get_view_url_of_note_model(note: &entity::note::Model, base_url: &Url) -> Url {
    if let Some(view_url) = &note.view_url {
        Url::parse(&view_url).expect("view_url should be valid")
    } else {
        base_url
            .join(&format!(
                "/client/note/{}",
                NoteID::from_db_trusted(note.id.clone())
            ))
            .unwrap()
    }
}
