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

use activitypub_federation::config::Data;
use activitypub_federation::traits::Object;
use identicon_rs::Identicon;
use sea_orm::EntityTrait;
use sea_orm::entity::*;
use tracing::warn;
use url::Url;

use crate::services::apub::UpdateActivity;
use crate::services::queue::QConn;
use crate::utils::sanitize::CleanString;
use crate::{MyFederationData, try_opt_res};

use crate::services::ServiceError;
use crate::services::follow::{get_follow_stats, is_following};

use super::super::MapToUnknown;
use super::super::db::MaybeTxConn;
use super::super::follow::FollowState;
use super::super::id::UploadID;
use super::super::kv::KVObject;
use super::super::note::get_user_note_count;
use super::super::{
    ServiceResult,
    db::Conn,
    id::{Identifier, UserID},
};
use super::is_blocking_user;
use super::{
    SimpleUserModel, UserSpecifier, get_apubuser_by_id, get_follower_inboxes, get_user_by_id,
    get_user_by_spec_with_remote, invalidate_user_cache,
};

/// ユーザープロフィールのアップデート情報。Some に設定された情報のみ変更される。
#[derive(Debug, Clone)]
pub struct UserProfileUpdate {
    pub nickname: Option<String>,
    pub bio: Option<String>,
    pub auto_follow_accept: Option<bool>,
    pub hide_follows: Option<bool>,
    pub avatar_upload_id: Option<Option<UploadID>>,
}

/// ユーザーのプロフィール情報を更新する。
/// ローカルデータベースに見つからない場合は何もしない。
pub async fn update_user_profile(
    conn: &Conn,
    rconn: &KVObject,
    qconn: &QConn,
    user_id: UserID,
    update: &UserProfileUpdate,
    base_url: &Url,
    data: &Data<MyFederationData>,
) -> ServiceResult<()> {
    let tx: MaybeTxConn = conn.as_tx().await?.into();

    let user = match entity::user::Entity::find_by_id(user_id.as_db())
        .one(&tx)
        .await
        .map_err_unknown()?
    {
        None => return Ok(()),
        Some(u) => u,
    };

    let mut user = user.into_active_model();
    if let Some(nickname) = update.nickname.as_ref() {
        user.nickname = Set(nickname.clone());
    }
    if let Some(bio) = update.bio.as_ref() {
        let cleaned = CleanString::clean(bio).into_inner();
        user.bio = Set(cleaned);
    }
    if let Some(auto_follow_accept) = update.auto_follow_accept {
        user.auto_follow_accept = Set(auto_follow_accept);
    }
    if let Some(hide_follows) = update.hide_follows {
        user.hide_follows = Set(hide_follows);
    }
    if let Some(avatar_upload_id) = update.avatar_upload_id.as_ref() {
        user.avatar = Set(avatar_upload_id.clone().map(|a| a.as_db()));
    }

    user.update(&tx).await.map_err_unknown()?;

    publish_user_profile_update_to_followers(&tx, qconn, user_id, base_url, data).await?;

    tx.commit().await?;

    invalidate_user_cache(rconn, user_id).await?;

    Ok(())
}

async fn publish_user_profile_update_to_followers(
    tx: &MaybeTxConn,
    qconn: &QConn,
    user_id: UserID,
    base_url: &Url,
    data: &Data<MyFederationData>,
) -> ServiceResult<()> {
    let user = get_apubuser_by_id(tx, user_id, base_url)
        .await?
        .ok_or_else(|| ServiceError::ise("updated user should exist"))?;

    if user.is_remote() {
        // update of remote user should no be published
        warn!("remote user update should not be published");
        return Ok(());
    }

    let follower_inboxes = get_follower_inboxes(tx, user_id).await?;

    let apub_user = user.clone().into_json(data).await?;
    let activity = UpdateActivity::from_user(apub_user);
    qconn
        .queue_activity(activity, user, follower_inboxes)
        .await?;

    Ok(())
}

/// ユーザーの詳細なプロフィール
#[derive(Debug, Clone)]
pub struct UserDetailedProfile {
    pub basic: SimpleUserModel,
    pub follow_count: u64,
    pub follower_count: u64,
    pub followable: bool,
    pub is_following: Option<FollowState>,
    pub is_followed: Option<FollowState>,
    pub is_me: bool,

    pub note_count: u64,

    pub is_blocked: Option<bool>,
    pub is_blocking: Option<bool>,

    pub url: Option<String>,
    pub view_url: Option<String>,

    pub auto_follow_accept: bool,
    pub hide_follows: bool,
}

async fn get_user_profile_impl(
    conn: &MaybeTxConn,
    viewer_id: Option<UserID>,
    user: SimpleUserModel,
) -> ServiceResult<Option<UserDetailedProfile>> {
    let is_viewer = match viewer_id {
        Some(v) => v == *user.id(),
        None => false,
    };

    let (is_following, is_followed) = match viewer_id {
        None => (None, None),
        Some(viewer_id) => {
            let following = is_following(conn, viewer_id, *user.id()).await?;
            let followed = is_following(conn, *user.id(), viewer_id).await?;
            (Some(following), Some(followed))
        }
    };

    let (is_blocking, is_blocked) = match viewer_id {
        None => (None, None),
        Some(viewer_id) => {
            let is_blocking = is_blocking_user(conn, viewer_id, user.id).await?;
            let is_blocked = is_blocking_user(conn, user.id, viewer_id).await?;
            (Some(is_blocking), Some(is_blocked))
        }
    };

    let follow_stats = get_follow_stats(conn, *user.id()).await?;

    let user_all = entity::user::Entity::find_by_id(user.id().as_db())
        .one(conn)
        .await
        .map_err_unknown()?;
    let user_all = match user_all {
        None => return Ok(None),
        Some(u) => u,
    };

    let note_count = get_user_note_count(conn, viewer_id, user.id, true).await?;

    Ok(Some(UserDetailedProfile {
        basic: user,
        is_following,
        is_followed,
        is_blocked,
        is_blocking,
        is_me: is_viewer,
        follow_count: follow_stats.following,
        follower_count: follow_stats.followers,
        followable: !is_viewer,
        note_count: note_count as u64,
        url: user_all.url,
        view_url: user_all.view_url,
        auto_follow_accept: user_all.auto_follow_accept,
        hide_follows: user_all.hide_follows,
    }))
}

/// ID を用いてユーザーの詳細なプロフィールを取得する。
/// ローカルデータベースのみ探索する。
/// viewer_id に値を設定すると、フォロー関係などを追加で取得する。
pub async fn get_user_profile_by_id(
    conn: &MaybeTxConn,
    rconn: &KVObject,
    viewer_id: Option<UserID>,
    user_id: UserID,
) -> ServiceResult<Option<UserDetailedProfile>> {
    let user = try_opt_res!(get_user_by_id(conn, rconn, user_id).await?);

    get_user_profile_impl(conn, viewer_id, user).await
}

/// ID を用いてユーザーの詳細なプロフィールを取得する。
/// ローカルデータベース上で見つからない場合は、リモートから取得する。
/// viewer_id に値を設定すると、フォロー関係などを追加で取得する。
pub async fn get_user_profile(
    conn: &MaybeTxConn,
    rconn: &KVObject,
    viewer_id: Option<UserID>,
    my_domain: &str,
    user_spec: &UserSpecifier,
    data: &Data<MyFederationData>,
) -> ServiceResult<Option<UserDetailedProfile>> {
    let user =
        try_opt_res!(get_user_by_spec_with_remote(conn, rconn, user_spec, my_domain, data).await?);

    get_user_profile_impl(conn, viewer_id, user).await
}

/// ユーザーアバター
#[derive(Debug)]
pub enum UserAvatar {
    Upload(UploadID),
    /// ideticon の jpeg データ
    Identicon(Vec<u8>),
}

/// ユーザーのアバター情報を取得する。
/// ローカルデータベースのみ探索する。
pub async fn get_user_avatar(
    conn: &MaybeTxConn,
    rconn: &KVObject,
    user_id: UserID,
) -> ServiceResult<Option<UserAvatar>> {
    let user = try_opt_res!(get_user_by_id(conn, rconn, user_id).await?);

    let avatar = match user.avatar() {
        Some(avatar_upload_id) => UserAvatar::Upload(avatar_upload_id.clone()),
        None => {
            let identicon = Identicon::new(&user_id.to_string());
            let img = identicon.export_jpeg_data().map_err_unknown()?;
            UserAvatar::Identicon(img)
        }
    };

    Ok(Some(avatar))
}
