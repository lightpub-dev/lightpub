use sea_orm::{Condition, EntityTrait, IntoActiveModel, Set, SqlErr, TryIntoModel};
use url::Url;

use crate::services::{
    MapToUnknown, ServiceError, ServiceResult,
    apub::{LikeActivity, LikeableObject, UndoActivity},
    db::Conn,
    id::{Identifier, NoteID, UserID},
    queue::QConn,
    user::get_apubuser_by_id,
};
use sea_orm::ActiveModelTrait;
use sea_orm::ColumnTrait;
use sea_orm::QueryFilter;

use super::{
    CalculateToAndCcResult, NoteLikeError, NoteSpecifier, calculate_to_and_cc,
    get::get_apubnote_by_spec, get_apubnote_by_id, note_visibility_check,
};

pub async fn note_like_add(
    conn: &Conn,
    qconn: &QConn,
    user_id: UserID,
    note_id: NoteID,
    is_private: bool,
    my_domain: &str,
    base_url: &Url,
) -> ServiceResult<()> {
    let tx = conn.as_tx().await?.into();

    let user = get_apubuser_by_id(&tx, user_id, base_url)
        .await?
        .ok_or(ServiceError::known(NoteLikeError::UserNotFound))?;

    // visibility check
    let ok = note_visibility_check(&tx, note_id, Some(user_id), false).await?;
    if !ok {
        return Err(ServiceError::known(NoteLikeError::NoteNotFound));
    }

    let like = entity::note_like::ActiveModel {
        note_id: Set(note_id.as_db()),
        user_id: Set(user_id.as_db()),
        is_private: Set(is_private as i8),
        ..Default::default()
    };
    let like = match like.save(&tx).await {
        Ok(li) => li.try_into_model().map_err_unknown()?,
        Err(e) => match e.sql_err() {
            Some(SqlErr::UniqueConstraintViolation(_)) => {
                // already liked
                return Ok(());
            }

            _ => return Err(ServiceError::unknown(e)),
        },
    };

    // ローカルユーザが like するとき activitypub 送信
    if user.is_local() && !is_private {
        let note = get_apubnote_by_spec(&tx, &NoteSpecifier::ID(note_id), my_domain, base_url)
            .await?
            .expect("note should exist");
        let like = LikeActivity::new(
            like.id,
            user.apub.url.clone(),
            LikeableObject::note(note.apub.url),
            base_url,
        );

        let CalculateToAndCcResult {
            inboxes,
            to: _,
            cc: _,
        } = calculate_to_and_cc(
            &tx,
            note_id,
            note.basic.author.id,
            note.basic.visibility,
            true,
            base_url,
        )
        .await?;

        qconn.queue_activity(like, user, inboxes).await?;
    }

    tx.commit().await?;
    Ok(())
}

pub async fn note_like_remove(
    conn: &Conn,
    qconn: &QConn,
    user_id: UserID,
    note_id: NoteID,
    is_private: bool,
    base_url: &Url,
) -> ServiceResult<()> {
    let tx = conn.as_tx().await?.into();

    let user = get_apubuser_by_id(&tx, user_id, base_url)
        .await?
        .ok_or(ServiceError::known(NoteLikeError::UserNotFound))?;

    // visibility check
    let ok = note_visibility_check(&tx, note_id, Some(user_id), false).await?;
    if !ok {
        return Err(ServiceError::known(NoteLikeError::NoteNotFound));
    }

    let like = entity::note_like::Entity::find()
        .filter(
            Condition::all()
                .add(entity::note_like::Column::NoteId.eq(note_id.as_db()))
                .add(entity::note_like::Column::UserId.eq(user_id.as_db()))
                .add(entity::note_like::Column::IsPrivate.eq(is_private)),
        )
        .one(&tx)
        .await
        .map_err_unknown()?;

    let like = match like {
        Some(i) => i,
        None => return Ok(()),
    };
    let like_id = like.id;
    let like = like.into_active_model();
    like.delete(&tx).await.map_err_unknown()?;

    // ローカルユーザが unlike するとき activitypub 送信
    if user.is_local() && !is_private {
        let note = get_apubnote_by_id(&tx, note_id, base_url, false)
            .await?
            .expect("note should exist");
        let like = LikeActivity::new(
            like_id,
            user.apub.url.clone(),
            LikeableObject::note(note.apub.url),
            base_url,
        );

        let CalculateToAndCcResult {
            to: _,
            cc: _,
            inboxes,
        } = calculate_to_and_cc(
            &tx,
            note_id,
            note.basic.author.id,
            note.basic.visibility,
            true,
            base_url,
        )
        .await?;

        let undo = UndoActivity::new(user.apub.url.clone(), like);
        qconn.queue_activity(undo, user, inboxes).await?;
    }

    tx.commit().await?;
    Ok(())
}
