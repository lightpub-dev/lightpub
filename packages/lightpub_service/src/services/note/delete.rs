use activitypub_federation::fetch::object_id::ObjectId;
use actix_web::http::StatusCode;
use chrono::Utc;
use expected_error_derive::ExpectedError;
use sea_orm::{Condition, EntityTrait, IntoActiveModel, Set};
use thiserror::Error;
use url::Url;

use crate::services::{
        MapToUnknown, ServiceError, ServiceResult,
        apub::{AnnounceActivity, DeleteActivity, UndoActivity},
        db::Conn,
        id::{Identifier, NoteID, UserID},
        queue::QConn,
        user::get_apubuser_by_id,
    };
use sea_orm::ActiveModelTrait;
use sea_orm::ColumnTrait;
use sea_orm::QueryFilter;

use super::{
    CalculateToAndCcResult, VisibilityModel, calculate_to_and_cc, calculate_to_and_cc_of_renote,
    get_apubnote_by_id, get_url_of_note_model,
};

#[derive(Debug, Clone, Error, ExpectedError)]
pub enum NoteDeleteError {
    #[error("Not allowed to delete note")]
    #[ee(status(StatusCode::FORBIDDEN))]
    NotAuthor,
    #[error("Renote cannot be deleted using this endpoint")]
    #[ee(status(StatusCode::BAD_REQUEST))]
    IsRenote,
}

pub async fn delete_note_by_id(
    conn: &Conn,
    qconn: &QConn,
    note_id: NoteID,
    viewer_id: UserID,
    base_url: &Url,
) -> ServiceResult<()> {
    delete_note_by_id_(conn, qconn, note_id, Some(viewer_id), base_url).await
}

pub async fn delete_note_by_id_(
    conn: &Conn,
    qconn: &QConn,
    note_id: NoteID,
    viewer_id: Option<UserID>, // None for no viewer check
    base_url: &Url,
) -> ServiceResult<()> {
    let txn = conn.as_tx().await?.into();
    let note = entity::note::Entity::find_by_id(note_id.as_db())
        .one(&txn)
        .await
        .map_err_unknown()?;

    let note = match note {
        None => return Ok(()),
        Some(n) => n,
    };

    if note.deleted_at.is_some() {
        // already deleted
        return Ok(());
    }

    // author check
    if let Some(viewer_id) = viewer_id {
        if UserID::from_db_trusted(note.author_id.clone()) != viewer_id {
            return Err(ServiceError::known(NoteDeleteError::NotAuthor));
        }
    }

    let note = if note.renote_of_id.is_some() {
        // renote is not supported by this function
        return Err(ServiceError::known(NoteDeleteError::IsRenote));
    } else {
        // normal note
        let mut active = note.into_active_model();
        active.deleted_at = Set(Some(Utc::now().fixed_offset()));
        active.update(&txn).await.map_err_unknown()?
    };

    // activitypub send
    if let Some(viewer_id) = viewer_id {
        let viewer_apub = get_apubuser_by_id(&txn, viewer_id, base_url)
            .await?
            .expect("viewer should exist");
        if viewer_apub.is_local() {
            let note_apub =
                get_apubnote_by_id(&txn, NoteID::from_db_trusted(note.id), base_url, true)
                    .await?
                    .expect("deleted should be stored in db");
            let delete = DeleteActivity::from_note(
                ObjectId::from(note_apub.apub.url.clone()),
                ObjectId::from(viewer_apub.apub.url.clone()),
                note.deleted_at.unwrap().to_utc(),
            );

            let CalculateToAndCcResult {
                to: _,
                cc: _,
                inboxes,
            } = calculate_to_and_cc(
                &txn,
                note_apub.basic.id,
                note_apub.basic.author.id,
                note_apub.basic.visibility,
                false,
                base_url,
            )
            .await?;

            qconn.queue_activity(delete, viewer_apub, inboxes).await?;
        }
    }

    txn.commit().await.map_err_unknown()?;

    Ok(())
}

pub async fn delete_renote_by_id(
    conn: &Conn,
    qconn: &QConn,
    target_note_id: NoteID,
    viewer_id: UserID,
    base_url: &Url,
) -> ServiceResult<()> {
    let txn = conn.as_tx().await?.into();
    let renote = entity::note::Entity::find()
        .filter(
            Condition::all()
                .add(entity::note::Column::AuthorId.eq(viewer_id.as_db()))
                .add(entity::note::Column::RenoteOfId.eq(target_note_id.as_db()))
                .add(entity::note::Column::Content.is_null())
                .add(entity::note::Column::DeletedAt.is_null()),
        )
        .one(&txn)
        .await
        .map_err_unknown()?;

    let renote = match renote {
        None => return Ok(()),
        Some(n) => n,
    };

    let renote_model = renote.clone();

    let renote = renote.into_active_model();
    renote.delete(&txn).await.map_err_unknown()?;

    // activitypub send
    let renote_author = get_apubuser_by_id(
        &txn,
        UserID::from_db_trusted(renote_model.author_id),
        base_url,
    )
    .await?
    .expect("renote author should exist");
    if renote_author.is_local() {
        let renote_apub_url = get_url_of_note_model(&renote_model, base_url);
        let renote_target_note = get_apubnote_by_id(
            &txn,
            NoteID::from_db_trusted(renote_model.renote_of_id.unwrap()),
            base_url,
            false,
        )
        .await?
        .expect("renote target should exist");
        let CalculateToAndCcResult { to, cc, inboxes } = calculate_to_and_cc_of_renote(
            &txn,
            UserID::from_db_trusted(renote_model.author_id),
            renote_target_note.basic.author.id,
            VisibilityModel::from_db(renote_model.visibility),
            base_url,
        )
        .await?;
        let announce = AnnounceActivity::from_note(
            ObjectId::from(renote_target_note.apub.url.clone()),
            renote_apub_url,
            ObjectId::from(renote_author.apub.url.clone()),
            to.into_iter().map(|u| ObjectId::from(u)).collect(),
            cc.into_iter().map(|u| ObjectId::from(u)).collect(),
            renote_model.created_at.to_utc(),
        );
        let undo = UndoActivity::new(renote_author.apub.url.clone(), announce);
        qconn.queue_activity(undo, renote_author, inboxes).await?;
    }

    txn.commit().await.map_err_unknown()?;

    Ok(())
}
