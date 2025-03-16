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

use actix_web::http::StatusCode;
use expected_error_derive::ExpectedError;
use sea_orm::ColumnTrait;
use sea_orm::PaginatorTrait;
use sea_orm::{Condition, EntityTrait, IntoActiveModel, QueryFilter, Set, SqlErr};
use serde::Serialize;
use thiserror::Error;
use url::Url;

use crate::services::apub::RejectActivity;
use crate::services::{MapToUnknown, ServiceError, id::Identifier};
use sea_orm::ActiveModelTrait;

use super::apub::AcceptActivity;
use super::apub::FollowActivity;
use super::apub::UndoActivity;
use super::db::MaybeTxConn;
use super::kv::KVObject;
use super::notification::NotificationBody;
use super::notification::add_notification;
use super::queue::QConn;
use super::user::get_apubuser_by_id;
use super::user::get_user_by_id;
use super::user::is_blocking_or_blocked;
use super::{ServiceResult, db::Conn, id::UserID};

#[derive(Debug, Error, ExpectedError)]
pub enum UserFollowError {
    #[error("User {0} does not exist")]
    #[ee(status(StatusCode::NOT_FOUND))]
    UserDoesNotExist(UserID),
    #[error("Cannot follow self")]
    #[ee(status(StatusCode::BAD_REQUEST))]
    SameUser,
    #[error("Cannot find pending follow")]
    #[ee(status(StatusCode::NOT_FOUND))]
    PendingFollowNotFound,
    #[error("Cannot find follow")]
    #[ee(status(StatusCode::NOT_FOUND))]
    FollowNotFound,
    #[error("User is blocking or blocked")]
    #[ee(status(StatusCode::BAD_REQUEST))]
    UserIsBlockingOrBlocked,
}

pub async fn follow_user(
    conn: &Conn,
    rconn: &KVObject,
    qconn: &QConn,
    follower_id: UserID,
    followee_id: UserID,
    base_url: &Url,
) -> ServiceResult<()> {
    if follower_id == followee_id {
        return Err(ServiceError::known(UserFollowError::SameUser));
    }

    let tx = conn.as_tx().await?.into();

    let follower = get_user_by_id(&tx, rconn, follower_id).await?;
    let follower = match follower {
        Some(f) => f,
        None => {
            return Err(ServiceError::known(UserFollowError::UserDoesNotExist(
                follower_id.clone(),
            )));
        }
    };

    let followee = get_user_by_id(&tx, rconn, followee_id).await?;
    let followee = match followee {
        Some(f) => f,
        None => {
            return Err(ServiceError::known(UserFollowError::UserDoesNotExist(
                followee_id.clone(),
            )));
        }
    };

    // ブロックしている or されている場合はエラー
    let block = is_blocking_or_blocked(&tx, follower.id, followee.id).await?;
    if block {
        return Err(ServiceError::known(
            UserFollowError::UserIsBlockingOrBlocked,
        ));
    }

    let model = entity::user_follow::ActiveModel {
        follower_id: Set(follower.id().as_db()),
        followed_id: Set(followee.id().as_db()),
        pending: Set(true as i8),
        ..Default::default()
    };
    let insert_result = model.insert(&tx).await;
    let insert_result = match insert_result {
        Ok(r) => r,
        Err(e) => match e.sql_err() {
            Some(SqlErr::UniqueConstraintViolation(_)) => {
                return Ok(()); // already following
            }
            _ => {
                return Err(ServiceError::unknown(e));
            }
        },
    };

    // send activitypub follow if follower is local and followee is remote
    if follower.is_local() && followee.is_remote() {
        let follower_model = get_apubuser_by_id(&tx, follower.id, base_url)
            .await?
            .expect("follower should exist");
        let followed_model = get_apubuser_by_id(&tx, followee.id, base_url)
            .await?
            .expect("followed should exist");

        let follower_url = follower_model.apub.url.clone();
        let followed_url = followed_model.apub.url.clone();
        let followed_inbox = followed_model.shared_inbox_or_inbox().clone();

        let follow = FollowActivity::new(insert_result.id, follower_url, followed_url, base_url);
        qconn
            .queue_activity(follow, follower_model, vec![followed_inbox])
            .await?;
    }

    let conn = tx.commit().await?;

    // check if followee is local and enable autoFollowAccept
    if followee.is_local() {
        let tx2: MaybeTxConn = conn.as_tx().await?.into();

        let followee_details = entity::user::Entity::find_by_id(followee.id().as_db())
            .one(&tx2)
            .await
            .map_err_unknown()?;
        if followee_details.is_some_and(|f| f.auto_follow_accept != 0) {
            accept_pending_follow_tx(&tx2, rconn, qconn, follower_id, followee_id, base_url)
                .await?;
            // add notification (followed)
            add_notification(
                &tx2,
                *followee.id(),
                &NotificationBody::Followed(follower.id().clone()),
            )
            .await?;
        } else {
            // add notification (follow requested)
            add_notification(
                &tx2,
                *followee.id(),
                &NotificationBody::FollowRequested(follower.id().clone()),
            )
            .await?;
        }

        tx2.commit().await?;
    }

    Ok(())
}

pub async fn unfollow_user(
    conn: &Conn,
    rconn: &KVObject,
    qconn: &QConn,
    follower_id: UserID,
    followee_id: UserID,
    base_url: &Url,
) -> ServiceResult<()> {
    if follower_id == followee_id {
        return Err(ServiceError::known(UserFollowError::SameUser));
    }

    let tx = conn.as_tx().await?.into();

    let follower = get_user_by_id(&tx, rconn, follower_id).await?;
    let follower = match follower {
        Some(f) => f,
        None => {
            return Err(ServiceError::known(UserFollowError::UserDoesNotExist(
                follower_id.clone(),
            )));
        }
    };

    let followee = get_user_by_id(&tx, rconn, followee_id).await?;
    let followee = match followee {
        Some(f) => f,
        None => {
            return Err(ServiceError::known(UserFollowError::UserDoesNotExist(
                followee_id.clone(),
            )));
        }
    };

    let model = entity::user_follow::Entity::find()
        .filter(
            Condition::all()
                .add(entity::user_follow::Column::FollowerId.eq(follower.id().as_db()))
                .add(entity::user_follow::Column::FollowedId.eq(followee.id().as_db())),
        )
        .one(&tx)
        .await
        .map_err_unknown()?;

    let model = match model {
        None => {
            tx.commit().await?;
            return Ok(());
        }
        Some(m) => m,
    };

    let follow_id = model.id;
    let follower_id = UserID::from_db_trusted(model.follower_id.clone());
    let followee_id = UserID::from_db_trusted(model.followed_id.clone());
    let follow_url = model.url.clone();

    // Unfollow
    let model = model.into_active_model();
    model.delete(&tx).await.map_err_unknown()?;

    // send activitypub undo if follower is local and followee is remote
    if follower.is_local() && followee.is_remote() {
        let follower_model = get_apubuser_by_id(&tx, follower_id, base_url)
            .await?
            .expect("follower should exist");
        let followed_model = get_apubuser_by_id(&tx, followee_id, base_url)
            .await?
            .expect("followed should exist");

        let follower_url = follower_model.apub.url.clone();
        let followed_url = followed_model.apub.url.clone();
        let followed_inbox = followed_model.shared_inbox_or_inbox().clone();

        assert!(follow_url.is_none()); // follow object is local, so no url
        let follow = FollowActivity::new(follow_id, follower_url.clone(), followed_url, base_url);
        let undo = UndoActivity::new(follower_url, follow);
        qconn
            .queue_activity(undo, follower_model, vec![followed_inbox])
            .await?;
    }

    tx.commit().await?;

    Ok(())
}

pub async fn accept_pending_follow(
    conn: &Conn,
    rconn: &KVObject,
    qconn: &QConn,
    follower_id: UserID,
    followee_id: UserID,
    base_url: &Url,
) -> ServiceResult<()> {
    let tx = conn.as_tx().await?.into();
    accept_pending_follow_tx(&tx, rconn, qconn, follower_id, followee_id, base_url).await?;
    tx.commit().await?;
    Ok(())
}

pub async fn accept_pending_follow_tx(
    tx: &MaybeTxConn,
    rconn: &KVObject,
    qconn: &QConn,
    follower_id: UserID,
    followee_id: UserID,
    base_url: &Url,
) -> ServiceResult<()> {
    let follower = match get_user_by_id(tx, rconn, follower_id).await? {
        Some(f) => f,
        None => {
            return Err(ServiceError::known(UserFollowError::UserDoesNotExist(
                follower_id.clone(),
            )));
        }
    };
    let followee = match get_user_by_id(&tx, rconn, followee_id).await? {
        Some(f) => f,
        None => {
            return Err(ServiceError::known(UserFollowError::UserDoesNotExist(
                followee_id.clone(),
            )));
        }
    };

    let follow = entity::user_follow::Entity::find()
        .filter(
            Condition::all()
                .add(entity::user_follow::Column::FollowerId.eq(follower_id.as_db()))
                .add(entity::user_follow::Column::FollowedId.eq(followee_id.as_db()))
                .add(entity::user_follow::Column::Pending.eq(true)),
        )
        .one(tx)
        .await
        .map_err_unknown()?;

    let follow = match follow {
        None => return Err(ServiceError::known(UserFollowError::PendingFollowNotFound)),
        Some(f) => f,
    };

    let follow_id = follow.id;
    let follow_url = follow.url.clone();

    let mut follow = follow.into_active_model();
    follow.pending = Set(false as i8);
    follow.save(tx).await.map_err_unknown()?;

    // if follower is remote and followee is local, send Accept
    if follower.is_remote() && followee.is_local() {
        let follower_model = get_apubuser_by_id(tx, follower.id, base_url)
            .await?
            .expect("follower should be exist");
        let followee_model = get_apubuser_by_id(tx, followee.id, base_url)
            .await?
            .expect("followee should be exist");

        let follower_url = follower_model.apub.url.clone();
        let followee_url = followee_model.apub.url.clone();
        let follower_inbox = follower_model.shared_inbox_or_inbox().clone();

        let follow = if let Some(follow_url) = follow_url {
            FollowActivity::new_from_url(
                Url::parse(&follow_url).expect("follow url should be valid"),
                follower_url,
                followee_url.clone(),
            )
        } else {
            FollowActivity::new(follow_id, follower_url, followee_url.clone(), base_url)
        };
        let accept = AcceptActivity::new(followee_url, follow);
        qconn
            .queue_activity(accept, followee_model, vec![follower_inbox])
            .await?;
    }

    Ok(())
}

pub async fn reject_pending_follow(
    conn: &Conn,
    rconn: &KVObject,
    qconn: &QConn,
    follower_id: UserID,
    followee_id: UserID,
    base_url: &Url,
) -> ServiceResult<()> {
    let tx = conn.as_tx().await?.into();

    let follower = match get_user_by_id(&tx, rconn, follower_id).await? {
        Some(f) => f,
        None => {
            return Err(ServiceError::known(UserFollowError::UserDoesNotExist(
                follower_id.clone(),
            )));
        }
    };
    let followee = match get_user_by_id(&tx, rconn, followee_id).await? {
        Some(f) => f,
        None => {
            return Err(ServiceError::known(UserFollowError::UserDoesNotExist(
                followee_id.clone(),
            )));
        }
    };

    let follow = entity::user_follow::Entity::find()
        .filter(
            Condition::all()
                .add(entity::user_follow::Column::FollowerId.eq(follower_id.as_db()))
                .add(entity::user_follow::Column::FollowedId.eq(followee_id.as_db())),
        )
        .one(&tx)
        .await
        .map_err_unknown()?;

    let follow = match follow {
        None => return Err(ServiceError::known(UserFollowError::FollowNotFound)),
        Some(f) => f,
    };

    let follow_id = follow.id;
    let follow_url = follow.url.clone();

    let follow = follow.into_active_model();
    follow.delete(&tx).await.map_err_unknown()?;

    // if follower is remote and followee is local, send Reject
    if follower.is_remote() && followee.is_local() {
        let follower_model = get_apubuser_by_id(&tx, follower_id, base_url)
            .await?
            .expect("follower should exist");
        let followed_model = get_apubuser_by_id(&tx, followee_id, base_url)
            .await?
            .expect("followed should exist");

        let follower_url = follower_model.apub.url.clone();
        let followed_url = followed_model.apub.url.clone();
        let follower_inbox = follower_model.shared_inbox_or_inbox().clone();

        let follow = if let Some(follow_url) = follow_url {
            FollowActivity::new_from_url(
                Url::parse(&follow_url).expect("follow url should be valid"),
                follower_url,
                followed_url.clone(),
            )
        } else {
            FollowActivity::new(follow_id, follower_url, followed_url.clone(), base_url)
        };
        let reject = RejectActivity::new(followed_url, follow);
        qconn
            .queue_activity(reject, followed_model, vec![follower_inbox])
            .await?;
    }

    tx.commit().await?;

    Ok(())
}

#[derive(Debug, Clone, Serialize, PartialEq, Eq, Copy)]
pub enum FollowState {
    Yes,
    Pending,
    No,
}

pub async fn is_following(
    conn: &MaybeTxConn,
    follower_id: UserID,
    followed_id: UserID,
) -> ServiceResult<FollowState> {
    let follow = entity::user_follow::Entity::find()
        .filter(
            Condition::all()
                .add(entity::user_follow::Column::FollowerId.eq(follower_id.as_db()))
                .add(entity::user_follow::Column::FollowedId.eq(followed_id.as_db())),
        )
        .one(conn)
        .await
        .map_err_unknown()?;

    match follow {
        Some(f) => {
            if f.pending != 0 {
                Ok(FollowState::Pending)
            } else {
                Ok(FollowState::Yes)
            }
        }
        None => Ok(FollowState::No),
    }
}

#[derive(Debug, Clone, Serialize)]
pub struct FollowStats {
    pub following: u64,
    pub followers: u64,
}

pub async fn get_follow_stats(conn: &MaybeTxConn, user_id: UserID) -> ServiceResult<FollowStats> {
    let following = entity::user_follow::Entity::find()
        .filter(
            Condition::all()
                .add(entity::user_follow::Column::FollowerId.eq(user_id.as_db()))
                .add(entity::user_follow::Column::Pending.eq(false)),
        )
        .count(conn)
        .await
        .map_err_unknown()?;
    let followers = entity::user_follow::Entity::find()
        .filter(
            Condition::all()
                .add(entity::user_follow::Column::FollowedId.eq(user_id.as_db()))
                .add(entity::user_follow::Column::Pending.eq(false)),
        )
        .count(conn)
        .await
        .map_err_unknown()?;

    Ok(FollowStats {
        following,
        followers,
    })
}
