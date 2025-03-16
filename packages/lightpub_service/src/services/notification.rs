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

use chrono::DateTime;
use chrono::Utc;
use expected_error::StatusCode;
use expected_error_derive::ExpectedError;
use sea_orm::ActiveModelTrait;
use sea_orm::ColumnTrait;
use sea_orm::Condition;
use sea_orm::EntityTrait;
use sea_orm::IntoActiveModel;
use sea_orm::PaginatorTrait;
use sea_orm::QueryFilter;
use sea_orm::QueryOrder;
use sea_orm::Set;
use sea_orm::TransactionTrait;
use serde::{Deserialize, Serialize};
use thiserror::Error;

use super::ServiceError;
use super::db::MaybeTxConn;
use super::id::NoteID;
use super::id::NotificationID;
use super::{
    MapToUnknown, ServiceResult,
    db::Conn,
    id::{Identifier, UserID},
};

#[derive(Debug, Clone, Serialize, Deserialize)]
pub enum NotificationBody {
    Followed(UserID),
    FollowRequested(UserID),
    FollowAccepted(UserID),
    Replied(UserID, NoteID, NoteID), // author_id, reply_note_id, replied_note_id
    Mentioned(UserID, NoteID),
    Renoted(UserID, NoteID),
}

#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct Notification {
    pub id: NotificationID,
    pub user_id: UserID,
    pub body: NotificationBody,
    pub created_at: DateTime<Utc>,
    pub read_at: Option<DateTime<Utc>>,
}

#[derive(Debug, Clone, Serialize, Deserialize, Error, ExpectedError)]
pub enum NotificationError {
    #[error("notification not found")]
    #[ee(status(StatusCode::NOT_FOUND))]
    NotOwner,
}

pub async fn get_notifications(conn: &Conn, user_id: UserID) -> ServiceResult<Vec<Notification>> {
    let notifications = entity::notification::Entity::find()
        .filter(entity::notification::Column::UserId.eq(user_id.as_db()))
        .order_by_desc(entity::notification::Column::CreatedAt)
        .all(conn.db())
        .await
        .map_err_unknown()?;

    let notifications = notifications
        .into_iter()
        .map(|f| Notification {
            id: NotificationID::from_db_trusted(f.id),
            user_id: UserID::from_db_trusted(f.user_id),
            body: serde_json::from_value(f.body).expect("invalid notification json"),
            created_at: f.created_at.to_utc(),
            read_at: f.read_at.map(|t| t.to_utc()),
        })
        .collect();
    Ok(notifications)
}

pub async fn add_notification(
    tx: &MaybeTxConn,
    user_id: UserID,
    body: &NotificationBody,
) -> ServiceResult<()> {
    let body_json = serde_json::to_value(body).map_err_unknown()?;

    let model = entity::notification::ActiveModel {
        user_id: Set(user_id.as_db()),
        body: Set(body_json),
        ..Default::default()
    };
    model.insert(tx).await.map_err_unknown()?;

    Ok(())
}

pub async fn mark_notification_read(
    conn: &Conn,
    notification_id: NotificationID,
    user_id: UserID,
) -> ServiceResult<()> {
    let txn = conn.db().begin().await.map_err_unknown()?;

    let model = entity::notification::Entity::find_by_id(notification_id.as_db())
        .one(&txn)
        .await
        .map_err_unknown()?;
    let model = match model {
        None => return Ok(()),
        Some(m) => m,
    };

    if UserID::from_db_trusted(model.user_id.clone()) != user_id {
        return Err(ServiceError::known(NotificationError::NotOwner));
    }

    let mut model = model.into_active_model();
    model.read_at = Set(Some(Utc::now().fixed_offset()));

    model.update(&txn).await.map_err_unknown()?;

    txn.commit().await.map_err_unknown()?;

    Ok(())
}

pub async fn mark_notification_read_all(conn: &Conn, user_id: UserID) -> ServiceResult<()> {
    let txn = conn.db().begin().await.map_err_unknown()?;

    entity::notification::Entity::update_many()
        .filter(entity::notification::Column::UserId.eq(user_id.as_db()))
        .set(entity::notification::ActiveModel {
            read_at: Set(Some(Utc::now().fixed_offset())),
            ..Default::default()
        })
        .exec(&txn)
        .await
        .map_err_unknown()?;

    txn.commit().await.map_err_unknown()?;

    Ok(())
}

pub async fn count_unread_notifications(conn: &Conn, user_id: UserID) -> ServiceResult<u64> {
    let count = entity::notification::Entity::find()
        .filter(
            Condition::all()
                .add(entity::notification::Column::UserId.eq(user_id.as_db()))
                .add(entity::notification::Column::ReadAt.is_null()),
        )
        .count(conn.db())
        .await
        .map_err_unknown()?;
    Ok(count)
}
