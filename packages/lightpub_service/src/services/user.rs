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
use activitypub_federation::fetch::object_id::ObjectId;
use activitypub_federation::fetch::webfinger::webfinger_resolve_actor;
use actix_web::http::StatusCode;
use derive_getters::Getters;
use expected_error_derive::ExpectedError;
use migration::{Expr, Query};
use sea_orm::{EntityTrait, StatementBuilder};
use sea_orm::{entity::*, query::*};
use serde::{Deserialize, Serialize};
use thiserror::Error;
use tracing::warn;
use url::Url;

use crate::MyFederationData;

use crate::services::ServiceError;

use super::db::MaybeTxConn;
use super::id::UploadID;
use super::kv::KVObject;
use super::{FederationServiceError, MapToUnknown};
use super::{
    ServiceResult,
    id::{Identifier, UserID},
};

mod apub;
mod block;
mod follow;
mod profile;
mod specifier;

pub use apub::{
    ApubUserEndpointsModel, ApubUserIconModel, ApubUserKind, ApubUserModel, UserApubData,
    UserWithApubModel,
};
pub use block::{block_user, is_blocking_or_blocked, is_blocking_user, unblock_user};
pub use follow::{UserFollow, get_user_followers, get_user_followings};
pub use profile::{
    UserAvatar, UserDetailedProfile, UserProfileUpdate, get_user_avatar, get_user_profile,
    get_user_profile_by_id, update_user_profile,
};
pub use specifier::UserSpecifier;

#[derive(Debug, Clone, Error, ExpectedError)]
pub enum UserGetError {
    #[error("Bad user specifier")]
    #[ee(status(StatusCode::BAD_REQUEST))]
    BadSpecifier,
    #[error("Bad URL")]
    #[ee(status(StatusCode::BAD_REQUEST))]
    BadURL,
}

async fn invalidate_user_cache(rconn: &KVObject, user_id: UserID) -> ServiceResult<()> {
    rconn.delete(format!("user:{user_id}")).await?;
    Ok(())
}

/// データベース上からユーザー情報を取得する。
/// ローカルデータベースのみ探索する。
pub async fn get_user_by_spec(
    conn: &MaybeTxConn,
    rconn: &KVObject,
    user_spec: &UserSpecifier,
    my_domain: &str,
) -> ServiceResult<Option<SimpleUserModel>> {
    let user_id = get_user_id_from_spec(conn, user_spec, my_domain).await?;
    match user_id {
        Some(id) => get_user_by_id(conn, rconn, id).await,
        None => Ok(None),
    }
}

/// データベース上またはリモートからユーザー情報を取得する。
pub async fn get_user_by_spec_with_remote(
    conn: &MaybeTxConn,
    rconn: &KVObject,
    user_spec: &UserSpecifier,
    my_domain: &str,
    data: &Data<MyFederationData>,
) -> ServiceResult<Option<SimpleUserModel>> {
    let local_user = get_user_by_spec(conn, rconn, user_spec, my_domain).await?;
    if let Some(user) = local_user {
        return Ok(Some(user));
    }

    // リモートから取得
    let user_spec = user_spec
        .clone()
        .try_parse_specifier(my_domain)
        .ok_or(ServiceError::known(UserGetError::BadSpecifier))?;

    match user_spec {
        UserSpecifier::ID(_) => {
            // ID ではリモート取得は不可能
            Ok(None)
        }
        UserSpecifier::Username(username, domain) => {
            // webfinger
            let identifier = if let Some(domain) = domain {
                format!("{}@{}", username, domain)
            } else {
                username.to_string()
            };
            let user = webfinger_resolve_actor::<_, UserWithApubModel>(&identifier, data).await;
            match user {
                Err(FederationServiceError::ServiceError(se)) => Err(se),
                Err(FederationServiceError::FederationError(
                    activitypub_federation::error::Error::WebfingerResolveFailed(e),
                )) => {
                    warn!("webfinger error: {e}");
                    Ok(None)
                }
                Err(e) => Err(e.into()),
                Ok(user) => Ok(Some(user.basic)),
            }
        }
        UserSpecifier::URL(url) => {
            let user_object_id = ObjectId::<UserWithApubModel>::from(url);
            let user = user_object_id.dereference(data).await;
            match user {
                Ok(user) => Ok(Some(user.basic)),
                Err(e) => {
                    warn!("remote user fetch error: {e}");
                    Ok(None)
                }
            }
        }
        _ => unreachable!("should be parsed"),
    }
}

/// データベース上からユーザー情報を取得する。
/// `get_user_by_spec` よりも詳細な情報を返す。
/// ローカルデータベースのみ探索する。
async fn get_apubuser_by_spec(
    conn: &MaybeTxConn,
    user_spec: &UserSpecifier,
    my_domain: &str,
    base_url: &Url,
) -> ServiceResult<Option<UserWithApubModel>> {
    let user_id = get_user_id_from_spec(conn, user_spec, my_domain).await?;
    match user_id {
        Some(id) => get_apubuser_by_id(conn, id, base_url).await,
        None => Ok(None),
    }
}

/// データベース上からユーザーを検索し、その ID を返す。
/// ローカルデータベースのみ探索する。
pub async fn get_user_id_from_spec(
    conn: &MaybeTxConn,
    user_spec: &UserSpecifier,
    my_domain: &str,
) -> ServiceResult<Option<UserID>> {
    let user_spec = user_spec
        .clone()
        .try_parse_specifier(my_domain)
        .ok_or(ServiceError::known(UserGetError::BadSpecifier))?;

    match user_spec {
        UserSpecifier::ID(id) => Ok(Some(id)),
        UserSpecifier::Username(username, domain) => {
            let domain = match domain {
                Some(domain) if domain == my_domain => None,
                Some(ref domain) => Some(domain.as_str()),
                None => None,
            };

            get_user_id_from_username(conn, &username, domain).await
        }
        UserSpecifier::URL(url) => get_user_id_from_url(conn, &url, my_domain).await,
        _ => unreachable!("should be parsed"),
    }
}

/// データベース上からユーザーを検索し、その ID を返す。
/// ローカルデータベースのみ探索する。
/// domain が None の場合、ローカルユーザーであることを示す。
async fn get_user_id_from_username(
    conn: &MaybeTxConn,
    username: &str,
    domain: Option<&str>,
) -> ServiceResult<Option<UserID>> {
    // try to find from local db
    let domain_str = domain.unwrap_or("");
    let user = entity::user::Entity::find()
        .filter(
            Condition::all()
                .add(entity::user::Column::Username.eq(username))
                .add(entity::user::Column::Domain.eq(domain_str)),
        )
        .one(conn)
        .await
        .map_err(ServiceError::unknown)?;

    if let Some(u) = user {
        return Ok(Some(UserID::from_db_trusted(u.id)));
    }

    Ok(None)
}

fn extract_local_user_id_from_url(url: &Url, my_domain: &str) -> ServiceResult<Option<UserID>> {
    // check if it's local
    let url_domain = url
        .domain()
        .ok_or(ServiceError::known(UserGetError::BadURL))?;
    if url_domain == my_domain {
        let path: Vec<_> = url.path().split("/").collect();
        if path.len() != 3 {
            return Err(ServiceError::known(UserGetError::BadURL));
        }
        if path[1] != "user" {
            return Err(ServiceError::known(UserGetError::BadURL));
        }
        let maybe_user_id = UserID::from_string(path[2]);
        Ok(maybe_user_id)
    } else {
        Ok(None) // remote URL
    }
}

#[test]
fn test_extract_local_user_id() {
    let user_id = UserID::new_random();
    let url = user_id.as_local_url(&Url::parse("https://example.com").unwrap());
    let extracted = extract_local_user_id_from_url(&url, "example.com").unwrap();
    assert_eq!(extracted, Some(user_id));
}

/// データベース上からユーザーを検索し、その ID を返す。
/// ローカルデータベースのみ探索する。
async fn get_user_id_from_url(
    conn: &MaybeTxConn,
    url: &Url,
    my_domain: &str,
) -> ServiceResult<Option<UserID>> {
    let local_user_id = extract_local_user_id_from_url(url, my_domain)?;
    if let Some(id) = local_user_id {
        return Ok(Some(id));
    }

    // try to find from local db
    let user = entity::user::Entity::find()
        .filter(entity::user::Column::Url.eq(url.to_string()))
        .one(conn)
        .await
        .map_err(ServiceError::unknown)?;

    if let Some(u) = user {
        return Ok(Some(UserID::from_db_trusted(u.id)));
    }

    Ok(None)
}

/// ユーザーの一般的な情報
#[derive(Debug, Clone, Getters, Serialize, Deserialize)]
#[serde(rename_all = "camelCase")]
pub struct SimpleUserModel {
    pub id: UserID,
    pub username: String,
    pub domain: Option<String>,
    pub nickname: String,
    pub bio: String,
    pub avatar: Option<UploadID>,
    pub specifier: String,
}

impl SimpleUserModel {
    pub fn is_local(&self) -> bool {
        self.domain.is_none()
    }

    pub fn is_remote(&self) -> bool {
        !self.is_local()
    }
}

/// ID を用いてユーザー情報を取得する。
/// ローカルデータベースのみ探索する。
pub async fn get_user_by_id(
    tx: &MaybeTxConn,
    rconn: &KVObject,
    id: UserID,
) -> ServiceResult<Option<SimpleUserModel>> {
    let cache = rconn.get(format!("user:{id}")).await?;
    if let Some(cache) = cache {
        return Ok(Some(cache));
    }

    let user = entity::user::Entity::find_by_id(id.as_db())
        .one(tx)
        .await
        .map_err(ServiceError::unknown)?;

    let user = match user {
        None => return Ok(None),
        Some(user) => user,
    };

    let avatar = user
        .avatar
        .as_ref()
        .map(|a| UploadID::from_db_trusted(a.clone()));

    let model = SimpleUserModel {
        id: UserID::from_db_trusted(user.id),
        username: user.username.clone(),
        domain: domain_to_optional(user.domain.clone()),
        nickname: user.nickname,
        bio: user.bio,
        avatar,
        specifier: UserSpecifier::username_and_domain(&user.username, &user.domain).to_string(),
    };

    rconn.set(format!("user:{id}"), &model).await?;

    Ok(Some(model))
}

/// ID を用いて詳細なユーザー情報を取得する。
/// ローカルデータベースのみ探索する。
pub async fn get_apubuser_by_id(
    tx: &MaybeTxConn,
    id: UserID,
    base_url: &Url,
) -> ServiceResult<Option<UserWithApubModel>> {
    let user = entity::user::Entity::find_by_id(id.as_db())
        .one(tx)
        .await
        .map_err(ServiceError::unknown)?;

    let user = match user {
        None => return Ok(None),
        Some(user) => user,
    };

    let avatar = user
        .avatar
        .as_ref()
        .map(|a| UploadID::from_db_trusted(a.clone()));

    let basic = SimpleUserModel {
        id,
        username: user.username.clone(),
        domain: domain_to_optional(user.domain.clone()),
        nickname: user.nickname,
        bio: user.bio,
        avatar,
        specifier: UserSpecifier::username_and_domain(&user.username, &user.domain).to_string(),
    };

    let build_url = |elem: &str| {
        base_url
            .join(&format!("user/{id}/{elem}"))
            .expect("failed to join URL")
    };

    let public_key = {
        if let Some(key) = user.public_key {
            key.clone()
        } else {
            // find from remote_public_key
            let pubkey = entity::remote_public_key::Entity::find()
                .filter(entity::remote_public_key::Column::OwnerId.eq(id.as_db()))
                .one(tx)
                .await
                .map_err_unknown()?;
            match pubkey {
                Some(key) => key.public_key,
                None => return Err(ServiceError::ise("public key not found")),
            }
        }
    };

    let is_remote = user.domain != "";
    let apub = UserApubData {
        url: user
            .url
            .map(|u| u.parse().expect("url is a url"))
            .unwrap_or_else(|| base_url.join(&format!("user/{}", id)).unwrap()),
        view_url: {
            if is_remote {
                user.view_url.map(|u| u.parse().expect("view_url is a url"))
            } else {
                Some(base_url.join(&format!("client/user/{}", id)).unwrap())
            }
        },
        inbox: user
            .inbox
            .map(|u| u.parse().expect("inbox is a url"))
            .unwrap_or_else(|| build_url("inbox")),
        outbox: user
            .outbox
            .map(|u| u.parse().expect("outbox is a url"))
            .unwrap_or_else(|| build_url("outbox")),
        shared_inbox: {
            if is_remote {
                user.shared_inbox
                    .map(|u| u.parse().expect("sharedInbox is a url"))
            } else {
                Some(base_url.join("inbox").unwrap())
            }
        },
        followers: {
            if is_remote {
                user.followers
                    .map(|u| u.parse().expect("followers is a url"))
            } else {
                Some(build_url("followers"))
            }
        },
        following: {
            if is_remote {
                user.following
                    .map(|u| u.parse().expect("following is a url"))
            } else {
                Some(build_url("following"))
            }
        },
        private_key: user.private_key.clone(),
        public_key,
        is_bot: user.is_bot != 0,
        auto_follow_accept: user.auto_follow_accept != 0,
        fetched_at: user.fetched_at.map(|d| d.and_utc()),
        created_at: user.created_at.map(|d| d.and_utc()),
    };

    Ok(Some(UserWithApubModel { basic, apub }))
}

#[test]
fn test_baseurl_join() {
    let base = Url::parse("https://example.com").unwrap();
    let joined = base.join("foo").unwrap();
    assert_eq!(joined, Url::parse("https://example.com/foo").unwrap());
}

#[test]
fn test_baseurl_tailslash_join() {
    let base = Url::parse("https://example.com/").unwrap();
    let joined = base.join("foo").unwrap();
    assert_eq!(joined, Url::parse("https://example.com/foo").unwrap());
}

pub fn domain_to_optional<S: AsRef<str>>(domain: S) -> Option<S> {
    if domain.as_ref() == "" {
        None
    } else {
        Some(domain)
    }
}

/// 対象ユーザーのフォロワーの全 shared_inbox または inbox を取得する。
/// 重複は省かれている。
async fn get_follower_inboxes(conn: &MaybeTxConn, user_id: UserID) -> ServiceResult<Vec<Url>> {
    use entity::user;
    use entity::user_follow;
    let query = Query::select()
        .column(user::Column::PreferredInbox)
        .from(user_follow::Entity)
        .inner_join(
            user::Entity,
            Expr::col((user_follow::Entity, user_follow::Column::FollowerId))
                .eq(Expr::col((user::Entity, user::Column::Id))),
        )
        .and_where(Expr::col(user_follow::Column::FollowedId).eq(user_id.as_db()))
        .and_where(Expr::col(user_follow::Column::Pending).eq(false))
        .and_where(Expr::col(user::Column::PreferredInbox).is_not_null())
        .distinct()
        .to_owned();

    let result = conn
        .query_all(StatementBuilder::build(
            &query,
            &conn.get_database_backend(),
        ))
        .await
        .map_err_unknown()?;

    let inboxes = result
        .into_iter()
        .map(|r| {
            let value: String = r
                .try_get("", user::Column::PreferredInbox.as_str())
                .unwrap();
            Url::parse(&value).expect("preferred_inbox is a url")
        })
        .collect();

    Ok(inboxes)
}

pub async fn get_total_users_count(tx: &MaybeTxConn) -> ServiceResult<u64> {
    let count = entity::user::Entity::find()
        .filter(entity::user::Column::Domain.eq(""))
        .count(tx)
        .await
        .map_err_unknown()?;

    Ok(count)
}

pub async fn is_admin(tx: &MaybeTxConn, user_id: UserID) -> ServiceResult<bool> {
    let user = entity::user::Entity::find_by_id(user_id.as_db())
        .one(tx)
        .await
        .map_err_unknown()?;

    let user = match user {
        Some(user) => user,
        None => return Ok(false),
    };

    Ok(user.is_admin != 0)
}
