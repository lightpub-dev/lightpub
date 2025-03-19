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
use push::PushNotificationBody;
use push::PushSendResult;
use push::WPClient;
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
use tracing::warn;
use url::Url;

use crate::try_opt_res;

use super::ServiceError;
use super::db::MaybeTxConn;
use super::id::NoteID;
use super::id::NotificationID;
use super::kv::KVObject;
use super::user::UserDetailedProfile;
use super::user::get_user_profile_by_id;
use super::{
    MapToUnknown, ServiceResult,
    db::Conn,
    id::{Identifier, UserID},
};

pub mod push;

#[derive(Debug, Clone, Serialize, Deserialize)]
pub enum NotificationBody {
    Followed(UserID),
    FollowRequested(UserID),
    FollowAccepted(UserID),
    Replied(UserID, NoteID, NoteID), // author_id, reply_note_id, replied_note_id
    Mentioned(UserID, NoteID),
    Renoted(UserID, NoteID),
}

#[derive(Debug, Clone)]
pub struct NotificationNoteData {
    pub id: NoteID,
    pub view_url: Url,
}

#[derive(Debug, Clone)]
pub enum NotificationBodyData {
    Followed(UserDetailedProfile),
    FollowRequested(UserDetailedProfile),
    FollowAccepted(UserDetailedProfile),
    Replied {
        author: UserDetailedProfile,
        reply_note: NotificationNoteData,
        replied_note: NotificationNoteData,
    },
    Mentioned {
        user: UserDetailedProfile,
        note: NotificationNoteData,
    },
    Renoted {
        user: UserDetailedProfile,
        renoted_note: NotificationNoteData,
    },
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
            body: serde_json::from_str(&f.body).expect("invalid notification json"),
            created_at: f.created_at.and_utc(),
            read_at: f.read_at.map(|t| t.and_utc()),
        })
        .collect();
    Ok(notifications)
}

pub async fn add_notification(
    tx: &MaybeTxConn,
    rconn: &KVObject,
    wp: Option<&WPClient>,
    user_id: UserID,
    body: &NotificationBody,
    base_url: &Url,
) -> ServiceResult<()> {
    let body_json = serde_json::to_string(body).map_err_unknown()?;

    let model = entity::notification::ActiveModel {
        user_id: Set(user_id.as_db()),
        body: Set(body_json),
        ..Default::default()
    };
    model.insert(tx).await.map_err_unknown()?;

    // Webpush notification
    if let Some(wp) = wp {
        if let Some(body) = get_related_notification_data(tx, rconn, base_url, body).await? {
            let body = PushNotificationBody::new_from_notification_body(base_url, body)?;
            let subs = push::get_subscriptions_for_user(tx, user_id).await?;
            for sub in subs {
                let result = wp.try_send(&sub.subscription, &body).await?;
                match result {
                    PushSendResult::Success => {}
                    PushSendResult::Failed(e) if e.should_disable_endpoint() => {
                        push::delete_subscription_id(tx, sub.subscription_id).await?;
                    }
                    PushSendResult::Failed(e) => {
                        warn!("Failed to send push notification: {:?}", e);
                    }
                }
            }
        }
    }

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
    model.read_at = Set(Some(Utc::now().naive_utc()));

    model.update(&txn).await.map_err_unknown()?;

    txn.commit().await.map_err_unknown()?;

    Ok(())
}

pub async fn mark_notification_read_all(conn: &Conn, user_id: UserID) -> ServiceResult<()> {
    let txn = conn.db().begin().await.map_err_unknown()?;

    entity::notification::Entity::update_many()
        .filter(entity::notification::Column::UserId.eq(user_id.as_db()))
        .set(entity::notification::ActiveModel {
            read_at: Set(Some(Utc::now().naive_utc())),
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

pub async fn get_related_notification_data(
    conn: &MaybeTxConn,
    rconn: &KVObject,
    base_url: &Url,
    body: &NotificationBody,
) -> ServiceResult<Option<NotificationBodyData>> {
    match body {
        NotificationBody::Followed(follower) => {
            let follower_model =
                try_opt_res!(get_user_profile_by_id(conn, rconn, None, *follower).await?);
            Ok(Some(NotificationBodyData::Followed(follower_model)))
        }
        NotificationBody::FollowRequested(follower) => {
            let follower_model =
                try_opt_res!(get_user_profile_by_id(conn, rconn, None, *follower).await?);
            Ok(Some(NotificationBodyData::FollowRequested(follower_model)))
        }
        NotificationBody::FollowAccepted(follow) => {
            let follow_model =
                try_opt_res!(get_user_profile_by_id(conn, rconn, None, *follow).await?);
            Ok(Some(NotificationBodyData::FollowAccepted(follow_model)))
        }
        NotificationBody::Mentioned(author_id, note_id) => {
            let author_model =
                try_opt_res!(get_user_profile_by_id(conn, rconn, None, *author_id).await?);
            Ok(Some(NotificationBodyData::Mentioned {
                user: author_model,
                note: NotificationNoteData {
                    id: *note_id,
                    view_url: base_url.join(&format!("/client/note/{note_id}")).unwrap(),
                },
            }))
        }
        NotificationBody::Replied(author_id, reply_note_id, replied_note_id) => {
            let author_model =
                try_opt_res!(get_user_profile_by_id(conn, rconn, None, *author_id).await?);
            Ok(Some(NotificationBodyData::Replied {
                author: author_model,
                reply_note: NotificationNoteData {
                    id: *reply_note_id,
                    view_url: base_url
                        .join(&format!("/client/note/{reply_note_id}"))
                        .unwrap(),
                },
                replied_note: NotificationNoteData {
                    id: *replied_note_id,
                    view_url: base_url
                        .join(&format!("/client/note/{replied_note_id}"))
                        .unwrap(),
                },
            }))
        }
        NotificationBody::Renoted(author_id, renoted_note_id) => {
            let author_model =
                try_opt_res!(get_user_profile_by_id(conn, rconn, None, *author_id).await?);
            Ok(Some(NotificationBodyData::Renoted {
                user: author_model,
                renoted_note: NotificationNoteData {
                    id: *renoted_note_id,
                    view_url: base_url
                        .join(&format!("/client/note/{renoted_note_id}"))
                        .unwrap(),
                },
            }))
        }
    }
}
