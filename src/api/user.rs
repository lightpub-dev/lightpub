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

use crate::api::federation::{apub_auth, handle_inbox};
use crate::api::note::create_parts_note_from_model;
/// Route handlers for /user
use crate::api::pagination::{ApubCollectionQuery, ApubPaginator, URLBasedPaginatedResponse};
use crate::template::{render_template, PartsNotes, PartsUserList, Template};
use crate::AppState;
use activitypub_federation::{protocol::context::WithContext, traits::Object};
use actix_multipart::form::text::Text as MpText;
use actix_multipart::form::{tempfile::TempFile, MultipartForm};
use actix_web::{get, patch, post, HttpRequest};
use actix_web::{
    http::{
        header::{CacheControl, CacheDirective, LOCATION},
        StatusCode,
    },
    web::{self, Path},
    HttpResponse, Responder,
};
use chrono::{DateTime, Utc};
use lightpub_service::services::note::{get_user_apub_outbox, get_user_note_count, get_user_notes};
use lightpub_service::services::user::{block_user, unblock_user};
use lightpub_service::services::{
    create_error_simple,
    follow::{accept_pending_follow, follow_user, reject_pending_follow, unfollow_user},
    id::UserID,
    upload::{save_upload_file, save_upload_file_info},
    user::{
        get_apubuser_by_id, get_user_avatar, get_user_followers, get_user_followings,
        update_user_profile, UserAvatar, UserProfileUpdate,
    },
    ServiceResult,
};
use percent_encoding::CONTROLS;
use serde::{Deserialize, Serialize};

#[derive(Debug, Serialize, Deserialize)]
struct GetUserNotesKey {
    bd: Option<DateTime<Utc>>,
}

#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct GetUserNotesQuery {
    pub key: Option<String>,
}

use super::note::create_user_list_from_models;
use super::RequestType;
use super::{auth::AuthedUser, pagination::Paginator, render_apub, FormBool};
use crate::api::auth::middleware_auth_jwt_optional;
use actix_web::middleware::from_fn;
#[get("/{user_id}/notes", wrap = "from_fn(middleware_auth_jwt_optional)")]
pub async fn api_get_user_notes(
    st: web::Data<AppState>,
    user_id: web::Path<UserID>,
    query: web::Query<GetUserNotesQuery>,
    auth: web::ReqData<AuthedUser>,
) -> ServiceResult<impl Responder> {
    let user_id = user_id.into_inner();

    let paginator = Paginator::new(
        20,
        |limit, key: Option<GetUserNotesKey>| {
            let conn = st.maybe_conn();
            let rconn = st.rconn().clone();
            let viewer_id = auth.user_id().clone();
            let user_id = user_id.clone();
            let st = st.clone();
            Box::pin(async move {
                let notes = get_user_notes(
                    &conn,
                    &rconn,
                    viewer_id,
                    user_id,
                    limit as u64,
                    key.as_ref().map(|k| k.bd).flatten(),
                )
                .await?;

                let mut note_vec = vec![];
                for n in notes {
                    note_vec.push(create_parts_note_from_model(&st, &n, viewer_id, None).await?);
                }
                Ok(note_vec)
            })
        },
        |n| GetUserNotesKey {
            bd: Some(n.note.created_at),
        },
    );

    let response = paginator
        .fetch_page(query.key.as_ref().map(|s| s.to_owned()))
        .await?;
    let paginated = URLBasedPaginatedResponse::from_paginated(response, move |k| {
        let new_query = GetUserNotesQuery { key: Some(k) };
        let qs = serde_qs::to_string(&new_query).unwrap();
        format!(
            "/user/{}/notes?{}",
            user_id,
            percent_encoding::utf8_percent_encode(&qs, CONTROLS)
        )
    });

    render_template(
        st.template(),
        &Template::PartsNotes(PartsNotes {
            authed: auth.is_authed().into(),
            data: paginated.data,
            next_url: paginated.next_url,
        }),
    )
}
use actix_middleware_etag::Etag;
#[get("/{user_id}/avatar", wrap = "Etag::default()")]
pub async fn api_get_user_avatar(
    st: web::Data<AppState>,
    user_id: web::Path<UserID>,
) -> ServiceResult<impl Responder> {
    let avatar = match get_user_avatar(&st.maybe_conn(), &st.rconn(), user_id.into_inner()).await? {
        None => return create_error_simple(StatusCode::NOT_FOUND, "user not found"),
        Some(a) => a,
    };

    match avatar {
        UserAvatar::Upload(id) => Ok(HttpResponse::TemporaryRedirect()
            .insert_header((LOCATION, format!("/upload/{}", id)))
            .insert_header(CacheControl(vec![
                CacheDirective::Public,
                CacheDirective::MaxAge(60),
                CacheDirective::MaxStale(60 * 60 * 24),
            ]))
            .finish()),
        UserAvatar::Identicon(img) => Ok(HttpResponse::Ok()
            .content_type("image/jpeg")
            .insert_header(CacheControl(vec![
                CacheDirective::Public,
                CacheDirective::MaxAge(60),
                CacheDirective::MaxStale(60 * 60 * 24),
            ]))
            .body(img)),
    }
}

#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct UserInteractionRequest {
    #[serde(rename = "type")]
    ty: UserInteractionType,
}

#[derive(Debug, Clone, Copy, PartialEq, Eq, Serialize, Deserialize)]
#[serde(rename_all = "camelCase")]
pub enum UserInteractionType {
    Follow,
    Unfollow,
    AcceptFollow,
    RejectFollow,
    Block,
    Unblock,
}
use crate::api::auth::middleware_auth_jwt_required;
#[post(
    "/{user_id}/interaction",
    wrap = "from_fn(middleware_auth_jwt_required)"
)]
pub async fn api_user_interaction(
    st: web::Data<AppState>,
    req: web::Json<UserInteractionRequest>,
    user_id: web::Path<UserID>,
    auth: web::ReqData<AuthedUser>,
) -> ServiceResult<impl Responder> {
    let my_id = auth.user_id_unwrap();
    let target_id = user_id.into_inner();

    // let data = st.request_data();
    use UserInteractionType::*;
    match req.ty {
        Follow => {
            follow_user(
                st.conn(),
                &st.rconn(),
                st.qconn(),
                st.wp(),
                my_id,
                target_id,
                st.base_url(),
            )
            .await?;
        }
        Unfollow => {
            unfollow_user(
                st.conn(),
                &st.rconn(),
                st.qconn(),
                my_id,
                target_id,
                st.base_url(),
            )
            .await?;
        }
        AcceptFollow => {
            accept_pending_follow(
                st.conn(),
                &st.rconn(),
                st.qconn(),
                target_id,
                my_id,
                st.base_url(),
            )
            .await?;
        }
        RejectFollow => {
            reject_pending_follow(
                st.conn(),
                &st.rconn(),
                st.qconn(),
                target_id,
                my_id,
                st.base_url(),
            )
            .await?;
        }
        Block => {
            block_user(&st.maybe_conn(), my_id, target_id).await?;
        }
        Unblock => {
            unblock_user(&st.maybe_conn(), my_id, target_id).await?;
        }
    }

    Ok(HttpResponse::Ok()
        .insert_header(("hx-refresh", "true"))
        .finish())
}

#[derive(Debug, MultipartForm)]
pub struct UserProfilePatch {
    #[multipart(limit = "10MB")]
    avatar: Option<TempFile>,
    #[multipart(rename = "avatarRemove")]
    avatar_remove: Option<MpText<FormBool>>,
    nickname: Option<MpText<String>>,
    bio: Option<MpText<String>>,
    #[multipart(rename = "autoFollowAccept")]
    auto_follow_accept: Option<MpText<FormBool>>,
    #[multipart(rename = "hideFollows")]
    hide_follows: Option<MpText<FormBool>>,
}

#[patch("/{user_id}/edit", wrap = "from_fn(middleware_auth_jwt_required)")]
pub async fn api_user_profile_update(
    st: web::Data<AppState>,
    auth: web::ReqData<AuthedUser>,
    mut req: MultipartForm<UserProfilePatch>,
) -> ServiceResult<impl Responder> {
    let user_id = auth.user_id_unwrap();

    let avatar_upload_id = if let Some(avatar) = req.avatar.take() {
        let (upload_id, filename, mime_type) = save_upload_file(avatar.file)?;
        // mime_type is already verified
        save_upload_file_info(
            &st.maybe_conn(),
            &upload_id,
            &filename,
            &mime_type.to_string(),
        )
        .await?;
        Some(upload_id)
    } else {
        None
    };
    let avatar_remove = req.avatar_remove.as_ref().map(|r| r.0 .0).unwrap_or(false);
    let avatar_upload_id = match (avatar_upload_id, avatar_remove) {
        (_, true) => Some(None),
        (Some(id), false) => Some(Some(id)),
        (None, false) => None,
    };

    let data = st.request_data();
    update_user_profile(
        st.conn(),
        &st.rconn(),
        st.qconn(),
        user_id,
        &UserProfileUpdate {
            nickname: req.nickname.as_ref().map(|n| n.0.clone()),
            bio: req.bio.as_ref().map(|n| n.0.clone()),
            auto_follow_accept: Some(
                req.auto_follow_accept
                    .as_ref()
                    .map(|a| a.0.value())
                    .unwrap_or(false),
            ),
            hide_follows: Some(
                req.hide_follows
                    .as_ref()
                    .map(|h| h.0.value())
                    .unwrap_or(false),
            ),
            avatar_upload_id,
        },
        st.base_url(),
        &data,
    )
    .await?;

    Ok(HttpResponse::Ok()
        .insert_header(("hx-redirect", "/client/my"))
        .finish())
}

#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct FollowListQuery {
    key: Option<String>,
}

#[get("/{user_id}/following", wrap = "from_fn(middleware_auth_jwt_optional)")]
pub async fn api_user_followings_list(
    st: web::Data<AppState>,
    user_id: web::Path<UserID>,
    query: web::Query<FollowListQuery>,
) -> ServiceResult<impl Responder> {
    let user_id = user_id.into_inner();

    let paginator = Paginator::new(
        20,
        |limit, key| {
            let conn = st.maybe_conn();
            let rconn = st.rconn();
            let user_id = user_id.clone();
            Box::pin(async move {
                let result =
                    get_user_followings(&conn, &rconn, &user_id, limit as u64, key).await?;
                let result = create_user_list_from_models(
                    result.into_iter().map(|f| (f.user, f.created_at)),
                )?;
                Ok(result)
            })
        },
        |last| last.created_at.unwrap(),
    );

    let result = paginator.fetch_page(query.key.clone()).await?;

    let result = URLBasedPaginatedResponse::from_paginated(result, |key| {
        let mut new_query = query.clone();
        new_query.key = Some(key);
        format!(
            "/user/{}/following?{}",
            user_id,
            serde_qs::to_string(&new_query.0).unwrap()
        )
    });

    let user_list = Template::PartsUserList(PartsUserList {
        data: result.data,
        next_url: result.next_url,
    });

    render_template(st.template(), &user_list)
}

#[get("/{user_id}/followers", wrap = "from_fn(middleware_auth_jwt_optional)")]
pub async fn api_user_followers_list(
    st: web::Data<AppState>,
    user_id: web::Path<UserID>,
    query: web::Query<FollowListQuery>,
) -> ServiceResult<impl Responder> {
    let user_id = user_id.into_inner();

    let paginator = Paginator::new(
        20,
        |limit, key| {
            let conn = st.maybe_conn();
            let rconn = st.rconn();
            let user_id = user_id.clone();
            Box::pin(async move {
                let result = get_user_followers(&conn, &rconn, &user_id, limit as u64, key).await?;
                let result = create_user_list_from_models(
                    result.into_iter().map(|f| (f.user, f.created_at)),
                )?;
                Ok(result)
            })
        },
        |last| last.created_at.unwrap(),
    );

    let result = paginator.fetch_page(query.key.clone()).await?;

    let result = URLBasedPaginatedResponse::from_paginated(result, |key| {
        let mut new_query = query.clone();
        new_query.key = Some(key);
        format!(
            "/user/{}/followers?{}",
            user_id,
            serde_qs::to_string(&new_query.0).unwrap()
        )
    });

    let user_list = Template::PartsUserList(PartsUserList {
        data: result.data,
        next_url: result.next_url,
    });

    render_template(st.template(), &user_list)
}

#[get("/{user_id}")]
pub async fn api_get_user(
    st: web::Data<AppState>,
    request_type: RequestType,
    user_id: Path<UserID>,
) -> ServiceResult<impl Responder> {
    match request_type {
        RequestType::APUB => {
            let data = st.request_data();
            let user =
                get_apubuser_by_id(&st.maybe_conn(), user_id.into_inner(), st.base_url()).await?;

            if let Some(user) = user {
                let user_json = user.into_json(&data).await?;
                Ok(render_apub(WithContext::new_default(user_json)))
            } else {
                Ok(HttpResponse::NotFound().finish())
            }
        }
        RequestType::HTML => Ok(HttpResponse::SeeOther()
            .insert_header(("location", format!("/client/user/{}", user_id.into_inner())))
            .finish()),
        _ => Ok(HttpResponse::NotAcceptable().finish()),
    }
}

#[post("/{user_id}/inbox")]
pub async fn api_user_inbox(
    st: web::Data<AppState>,
    req: HttpRequest,
    body: web::Bytes,
) -> ServiceResult<impl Responder> {
    handle_inbox(st, req, body, true).await
}

#[derive(Debug, Clone, Serialize, Deserialize)]
struct UserOutboxKey {
    bd: Option<DateTime<Utc>>,
}

#[get("/{user_id}/outbox")]
pub async fn api_user_outbox(
    st: web::Data<AppState>,
    req: HttpRequest,
    user_id: web::Path<UserID>,
    query: web::Query<ApubCollectionQuery>,
    body: web::Bytes,
) -> ServiceResult<impl Responder> {
    let user_id = user_id.into_inner();
    let endpoint = st
        .base_url()
        .join(format!("user/{}/outbox", user_id).as_str())
        .unwrap();

    let data = st.request_data();
    let viewer_id = apub_auth(&req, body, &data).await?.map(|v| v.basic.id);

    let paginator = ApubPaginator::new(
        endpoint,
        20,
        |limit, key: Option<UserOutboxKey>| {
            let conn = st.maybe_conn();
            let viewer_id = viewer_id.clone();
            let user_id = user_id.clone();
            let base_url = st.base_url().clone();
            let data = st.request_data();
            Box::pin(async move {
                let notes = get_user_apub_outbox(
                    &conn,
                    viewer_id,
                    user_id,
                    limit as u64,
                    key.as_ref().map(|k| k.bd).flatten(),
                    &base_url,
                    &data,
                )
                .await?;

                Ok(notes)
            })
        },
        |n| UserOutboxKey {
            bd: Some(n.published()),
        },
        || {
            let conn = st.maybe_conn();
            let viewer_id = viewer_id.clone();
            Box::pin(async move {
                let count = get_user_note_count(&conn, viewer_id, user_id, false).await?;
                Ok(count as usize)
            })
        },
    );

    let query = query.into_inner();
    let res = paginator.response(&query).await?;
    Ok(render_apub(WithContext::new_default(res)))
}
