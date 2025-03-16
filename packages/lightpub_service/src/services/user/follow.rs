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

use chrono::{DateTime, Utc};
use sea_orm::EntityTrait;
use sea_orm::{entity::*, query::*};
use serde::Serialize;

use crate::services::user::get_user_by_id;

use super::super::MapToUnknown;
use super::super::db::MaybeTxConn;
use super::super::kv::KVObject;
use super::super::{
    ServiceResult,
    id::{Identifier, UserID},
};
use super::SimpleUserModel;

/// ユーザーのフォロー・フォロワー情報
#[derive(Debug, Clone, Serialize)]
#[serde(rename_all = "camelCase")]
pub struct UserFollow {
    pub created_at: DateTime<Utc>,
    pub user: SimpleUserModel,
}

/// ユーザーのフォロワー一覧を取得する。
/// 対象ユーザーがローカルデータベース上に存在しない場合、空配列を返す。
pub async fn get_user_followers(
    conn: &MaybeTxConn,
    rconn: &KVObject,
    user_id: &UserID,
    limit: u64,
    before_date: Option<DateTime<Utc>>,
) -> ServiceResult<Vec<UserFollow>> {
    use entity::user_follow::Column;
    let followers = entity::user_follow::Entity::find()
        .filter(
            Condition::all()
                .add(Column::FollowedId.eq(user_id.as_db()))
                .add(Column::Pending.eq(false))
                .add_option(before_date.map(|d| Column::CreatedAt.lte(d))),
        )
        .limit(limit)
        .order_by_desc(entity::user_follow::Column::CreatedAt)
        .all(conn)
        .await
        .map_err_unknown()?;
    // TODO: omit "hideFollows" user

    let mut users = Vec::new();
    for model in followers {
        let follower = get_user_by_id(
            conn,
            rconn,
            UserID::from_db_trusted(model.follower_id.clone()),
        )
        .await?;
        if let Some(f) = follower {
            users.push(UserFollow {
                user: f,
                created_at: model.created_at.to_utc(),
            });
        }
    }

    Ok(users)
}

/// ユーザーのフォロー一覧を取得する。
/// 対象ユーザーがローカルデータベース上に存在しない場合、空配列を返す。
pub async fn get_user_followings(
    conn: &MaybeTxConn,
    rconn: &KVObject,
    user_id: &UserID,
    limit: u64,
    before_date: Option<DateTime<Utc>>,
) -> ServiceResult<Vec<UserFollow>> {
    use entity::user_follow::Column;
    let followers = entity::user_follow::Entity::find()
        .filter(
            Condition::all()
                .add(Column::FollowerId.eq(user_id.as_db()))
                .add(Column::Pending.eq(false))
                .add_option(before_date.map(|d| Column::CreatedAt.lte(d))),
        )
        .limit(limit)
        .order_by_desc(entity::user_follow::Column::CreatedAt)
        .all(conn)
        .await
        .map_err_unknown()?;
    // TODO: omit "hideFollows" user

    let mut users = Vec::new();
    for model in followers {
        let follower = get_user_by_id(
            conn,
            rconn,
            UserID::from_db_trusted(model.followed_id.clone()),
        )
        .await?;
        if let Some(f) = follower {
            users.push(UserFollow {
                user: f,
                created_at: model.created_at.to_utc(),
            });
        }
    }

    Ok(users)
}
