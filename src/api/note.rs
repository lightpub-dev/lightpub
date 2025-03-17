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

use crate::api::auth::middleware_auth_jwt_optional;
use crate::api::auth::middleware_auth_jwt_required;
use activitypub_federation::protocol::context::WithContext;
use activitypub_federation::traits::Object;
use actix_multipart::form::{tempfile::TempFile, text::Text as MpText, MultipartForm};
use actix_web::middleware::from_fn;
use actix_web::{
    body::BoxBody, delete, get, http::StatusCode, patch, post, put, web, HttpRequest, HttpResponse,
    Responder,
};
use chrono::{DateTime, Utc};
use lightpub_service::services::user::UserSpecifier;
use percent_encoding::CONTROLS;
use serde::{Deserialize, Serialize};

use crate::{
    api::{pagination::Paginator, APIResponseBuilder},
    template::{
        render_template, EditNoteContentData, EditNoteData, NoteAuthorData, NoteContentData,
        PartsEditNote, PartsNote, PartsNoteNote, PartsNotes, PartsUserList, PartsUserListData,
        PartsUserListDataUser, RenoteInfo, RenoteInfoUser, Template,
    },
    AppState,
};
use lightpub_service::services::{
    create_error_simple,
    id::{NoteID, UserID},
    note::{
        create_note, create_renote, delete_note_by_id, delete_renote_by_id, edit_note,
        get_apubnote_by_id_visibility_check, get_liked_users, get_note_by_id_visibility_check,
        get_note_replies, get_renoted_users, note_like_add, note_like_remove, ContentType,
        DetailedNoteModel, NoteUpload, PostCreateOptionsBuilder, VisibilityModel,
    },
    upload::save_upload_file,
    user::{get_user_by_id, SimpleUserModel},
    ServiceResult, UpsertOperation,
};

use super::{
    auth::AuthedUser, federation::apub_auth, pagination::URLBasedPaginatedResponse, render_apub,
    APIResponse, FormBool, RequestType,
};

#[derive(Debug, Deserialize, Clone, Copy)]
#[serde(rename_all = "snake_case")]
pub enum CreatableNoteContentType {
    Plain,
    Md,
    Latex,
}

impl CreatableNoteContentType {
    pub fn into_content_type(self) -> ContentType {
        use CreatableNoteContentType::*;
        match self {
            Plain => ContentType::Plain,
            Md => ContentType::Md,
            Latex => ContentType::Latex,
        }
    }
}

#[derive(Debug, Deserialize, Clone, Copy)]
#[serde(rename_all = "snake_case")]
pub enum CreatableNoteVisibility {
    Public,
    Unlisted,
    Follower,
    Private,
}

impl CreatableNoteVisibility {
    pub fn into_visibility(self) -> VisibilityModel {
        use CreatableNoteVisibility::*;
        match self {
            Public => VisibilityModel::Public,
            Unlisted => VisibilityModel::Unlisted,
            Follower => VisibilityModel::Follower,
            Private => VisibilityModel::Private,
        }
    }
}

#[derive(Debug, MultipartForm)]
pub struct CreateNoteRequest {
    content: MpText<String>,
    #[multipart(rename = "contentType")]
    content_type: MpText<CreatableNoteContentType>,
    sensitive: Option<MpText<FormBool>>,
    visibility: MpText<CreatableNoteVisibility>,
    #[multipart(rename = "replyToId")]
    reply_to_id: Option<MpText<NoteID>>,
    #[multipart(limit = "10MB")]
    file: Vec<TempFile>,
}

#[derive(Debug, Clone, Serialize)]
#[serde(rename_all = "camelCase")]
pub struct CreateNoteResponse {
    note_id: NoteID,
}

#[post("", wrap = "from_fn(middleware_auth_jwt_required)")]
pub async fn api_create_note(
    st: web::Data<AppState>,
    auth: web::ReqData<AuthedUser>,
    MultipartForm(mut req): MultipartForm<CreateNoteRequest>,
) -> ServiceResult<APIResponse<CreateNoteResponse>> {
    let mut uploads = vec![];
    let req_files = std::mem::replace(&mut req.file, vec![]);
    for upload in req_files {
        let (upload_id, filename, mime_type) = save_upload_file(upload.file)?;
        uploads.push(NoteUpload::File(upload_id, filename, mime_type.to_string()));
    }

    let data = st.request_data();
    let result = create_note(
        st.conn(),
        &st.rconn(),
        st.qconn(),
        auth.user_id().expect("must auth"),
        &req.content,
        req.content_type.0.into_content_type(),
        req.visibility.0.into_visibility(),
        &PostCreateOptionsBuilder::default()
            .sensitive(UpsertOperation::Set(
                req.sensitive.map(|s| s.0 .0).unwrap_or(false),
            ))
            .reply_to_id(UpsertOperation::Set(req.reply_to_id.map(|r| r.0)))
            .uploads(UpsertOperation::Set(uploads))
            .build()
            .unwrap(),
        &st.my_domain(),
        st.base_url(),
        &data,
    )
    .await?;

    Ok(APIResponseBuilder::default()
        .data(CreateNoteResponse { note_id: result })
        .do_refresh(true)
        .build()
        .unwrap())
}

#[derive(Debug, Clone, Deserialize)]
pub struct CreateRenoteRequest {
    visibility: CreatableNoteVisibility,
}

#[post("/{note_id}/renote", wrap = "from_fn(middleware_auth_jwt_required)")]
pub async fn api_create_renote(
    st: web::Data<AppState>,
    auth: web::ReqData<AuthedUser>,
    note_id: web::Path<NoteID>,
    req: web::Json<CreateRenoteRequest>,
) -> ServiceResult<APIResponse<CreateNoteResponse>> {
    let note_id = note_id.into_inner();
    // let data = st.request_data();
    let renote_id = create_renote(
        st.conn(),
        &st.rconn(),
        st.qconn(),
        auth.user_id_unwrap(),
        note_id,
        req.visibility.into_visibility(),
        st.base_url(),
    )
    .await?;

    Ok(APIResponseBuilder::default()
        .data(CreateNoteResponse { note_id: renote_id })
        .trigger_event("note-refresh")
        .build()
        .unwrap())
}

pub async fn create_parts_note_from_model(
    st: &AppState,
    dnote: &DetailedNoteModel,
    viewer_id: Option<UserID>,
    renoted_by: Option<UserID>,
) -> ServiceResult<PartsNote> {
    let note = &dnote.basic;
    let content = match note.content.as_ref() {
        None => None,
        Some(content) => Some(NoteContentData {
            content: content.render_to_html(st.qconn()).await?,
        }),
    };

    let renote_info = if let Some(renoter) = renoted_by {
        let renoter = get_user_by_id(&st.maybe_conn(), &st.rconn(), renoter).await?;
        match renoter {
            None => None,
            Some(renoter) => Some(RenoteInfo {
                user: RenoteInfoUser {
                    nickname: renoter.nickname,
                    specifier: renoter.specifier,
                },
            }),
        }
    } else {
        None
    };

    Ok(PartsNote {
        renote_info,
        authed: viewer_id.is_some().into(),
        note: PartsNoteNote {
            id: note.id,
            author: NoteAuthorData {
                id: note.author.id,
                nickname: note.author.nickname.clone(),
                specifier: note.author.specifier(),
            },
            content,
            visibility: note.visibility,
            created_at: note.created_at,
            sensitive: note.sensitive,
            reply_to_id: note.reply_to_id,
            renote_of_id: note.renote_of_id,
            view_url: None,
            liked: dnote.details.liked.unwrap_or(false),
            bookmarked: dnote.details.bookmarked.unwrap_or(false),
            uploads: if note.uploads.len() == 0 {
                None
            } else {
                Some(
                    note.uploads
                        .iter()
                        .map(|u| {
                            let upload_id = u.data().upload_id();
                            format!("/upload/{}", upload_id)
                        })
                        .collect(),
                )
            },
            is_my_note: viewer_id.map(|v| v == note.author.id).unwrap_or(false),
            renotable: note.is_renotable(),
            renoted: dnote.details.renoted,
            reply_count: dnote.details.reply_count,
            renote_count: dnote.details.renote_count,
            like_count: dnote.details.like_count,
        },
    })
}

#[derive(Debug, Clone, Deserialize)]
#[serde(rename_all = "camelCase")]
pub struct GetNoteQuery {
    renoted_by: Option<UserID>,
}

async fn get_apub_note(
    st: &AppState,
    note_id: NoteID,
    req: &HttpRequest,
    body: web::Bytes,
) -> ServiceResult<HttpResponse<BoxBody>> {
    let data = st.request_data();
    let actor = apub_auth(req, body, &data).await?;

    let note = get_apubnote_by_id_visibility_check(
        &st.maybe_conn(),
        note_id,
        actor.map(|a| a.basic.id),
        st.base_url(),
        false,
    )
    .await?;

    match note {
        None => Ok(HttpResponse::NotFound().finish()),
        Some(note) => Ok(render_apub(WithContext::new_default(
            note.into_json(&data).await?,
        ))),
    }
}

#[get("/{note_id}", wrap = "from_fn(middleware_auth_jwt_optional)")]
pub async fn api_get_note(
    st: web::Data<AppState>,
    request_type: RequestType,
    note_id: web::Path<(NoteID,)>,
    auth: web::ReqData<AuthedUser>,
    query: web::Query<GetNoteQuery>,
    req: HttpRequest,
    body: web::Bytes,
) -> ServiceResult<impl Responder> {
    let note_id = note_id.into_inner().0;

    match request_type {
        RequestType::APUB => {
            let st = st.into_inner();
            return get_apub_note(&st, note_id, &req, body).await;
        }
        RequestType::HTML => {
            return Ok(HttpResponse::SeeOther()
                .insert_header(("location", format!("/client/note/{note_id}")))
                .finish());
        }
        RequestType::HTMX => {}
    }

    let note = get_note_by_id_visibility_check(
        &st.maybe_conn(),
        &st.rconn(),
        note_id,
        auth.user_id(),
        false,
    )
    .await?;

    match note {
        None => return create_error_simple(StatusCode::NOT_FOUND, "note not found"),
        Some(note) => {
            let data =
                create_parts_note_from_model(&st, &note, auth.user_id(), query.renoted_by).await?;

            render_template(st.template(), &Template::PartsNote(data))
        }
    }
}

#[get("/{note_id}/edit", wrap = "from_fn(middleware_auth_jwt_required)")]
pub async fn api_edit_note_view(
    st: web::Data<AppState>,
    note_id: web::Path<NoteID>,
    auth: web::ReqData<AuthedUser>,
) -> ServiceResult<impl Responder> {
    let note_id = note_id.into_inner();
    let note = get_note_by_id_visibility_check(
        &st.maybe_conn(),
        &st.rconn(),
        note_id,
        auth.user_id(),
        false,
    )
    .await?;

    match note {
        None => return create_error_simple(StatusCode::NOT_FOUND, "note not found"),
        Some(note) => {
            // owner check
            if note.basic.author.id != auth.user_id_unwrap() {
                return create_error_simple(StatusCode::FORBIDDEN, "not your note");
            }

            let content = match note.basic.content {
                None => return create_error_simple(StatusCode::BAD_REQUEST, "cannot edit renotes"),
                Some(n) => EditNoteContentData {
                    content: n.as_raw_text().to_owned(),
                    content_type: n.content_type(),
                },
            };

            let edit_data = Template::PartsEditNote(PartsEditNote {
                note: EditNoteData {
                    content,
                    id: note.basic.id,
                },
            });

            render_template(st.template(), &edit_data)
        }
    }
}

#[derive(Debug, Clone, Serialize, Deserialize)]
struct UserOutboxKey {
    bd: Option<DateTime<Utc>>,
}
#[derive(Debug, Serialize, Deserialize)]
struct GetUserNotesKey {
    bd: Option<DateTime<Utc>>,
}

#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct GetUserNotesQuery {
    pub key: Option<String>,
}

#[get("/{note_id}/replies", wrap = "from_fn(middleware_auth_jwt_optional)")]
pub async fn api_get_replies(
    st: web::Data<AppState>,
    note_id: web::Path<NoteID>,
    auth: web::ReqData<AuthedUser>,
    query: web::Query<GetUserNotesQuery>,
) -> ServiceResult<impl Responder> {
    let note_id = note_id.into_inner();

    let paginator = Paginator::new(
        20,
        |limit, key: Option<GetUserNotesKey>| {
            let conn = st.maybe_conn();
            let rconn = st.rconn().clone();
            let viewer_id = auth.user_id().clone();
            let st = st.clone();
            Box::pin(async move {
                let notes = get_note_replies(
                    &conn,
                    &rconn,
                    viewer_id,
                    note_id,
                    limit as u64,
                    key.map(|k| k.bd).flatten(),
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
            "/note/{}/replies?{}",
            note_id,
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

#[put("/{note_id}/like", wrap = "from_fn(middleware_auth_jwt_required)")]
pub async fn api_note_add_like(
    st: web::Data<AppState>,
    note_id: web::Path<NoteID>,
    auth: web::ReqData<AuthedUser>,
) -> ServiceResult<impl Responder> {
    let user_id = auth.user_id_unwrap();
    let note_id = note_id.into_inner();

    // let data = st.request_data();
    note_like_add(
        st.conn(),
        st.qconn(),
        user_id,
        note_id,
        false,
        &st.my_domain(),
        st.base_url(),
    )
    .await?;

    Ok(APIResponseBuilder::default()
        .data(())
        .trigger_event("note-refresh")
        .build()
        .unwrap())
}

#[put("/{note_id}/bookmark", wrap = "from_fn(middleware_auth_jwt_required)")]
pub async fn api_note_add_bookmark(
    st: web::Data<AppState>,
    note_id: web::Path<NoteID>,
    auth: web::ReqData<AuthedUser>,
) -> ServiceResult<impl Responder> {
    let user_id = auth.user_id_unwrap();
    let note_id = note_id.into_inner();

    // let data = st.request_data();
    note_like_add(
        st.conn(),
        st.qconn(),
        user_id,
        note_id,
        true,
        &st.my_domain(),
        st.base_url(),
    )
    .await?;

    Ok(APIResponseBuilder::default()
        .data(())
        .trigger_event("note-refresh")
        .build()
        .unwrap())
}

#[delete("/{note_id}/like", wrap = "from_fn(middleware_auth_jwt_required)")]
pub async fn api_note_remove_like(
    st: web::Data<AppState>,
    note_id: web::Path<NoteID>,
    auth: web::ReqData<AuthedUser>,
) -> ServiceResult<impl Responder> {
    let user_id = auth.user_id_unwrap();
    let note_id = note_id.into_inner();

    // let data = st.request_data();
    note_like_remove(
        st.conn(),
        st.qconn(),
        user_id,
        note_id,
        false,
        st.base_url(),
    )
    .await?;

    Ok(APIResponseBuilder::default()
        .data(())
        .trigger_event("note-refresh")
        .build()
        .unwrap())
}

#[delete("/{note_id}/bookmark", wrap = "from_fn(middleware_auth_jwt_required)")]
pub async fn api_note_remove_bookmark(
    st: web::Data<AppState>,
    note_id: web::Path<NoteID>,
    auth: web::ReqData<AuthedUser>,
) -> ServiceResult<impl Responder> {
    let user_id = auth.user_id_unwrap();
    let note_id = note_id.into_inner();

    // let data = st.request_data();
    note_like_remove(st.conn(), st.qconn(), user_id, note_id, true, st.base_url()).await?;

    Ok(APIResponseBuilder::default()
        .data(())
        .trigger_event("note-refresh")
        .build()
        .unwrap())
}

#[delete("/{note_id}", wrap = "from_fn(middleware_auth_jwt_required)")]
pub async fn api_note_delete(
    st: web::Data<AppState>,
    note_id: web::Path<NoteID>,
    auth: web::ReqData<AuthedUser>,
) -> ServiceResult<impl Responder> {
    let user_id = auth.user_id_unwrap();
    let note_id = note_id.into_inner();

    // let data = st.request_data();
    delete_note_by_id(st.conn(), st.qconn(), note_id, user_id, st.base_url()).await?;

    Ok(HttpResponse::NoContent()
        .insert_header(("hx-refresh", "true"))
        .finish())
}

#[delete("/{note_id}/renote", wrap = "from_fn(middleware_auth_jwt_required)")]
pub async fn api_note_delete_by_renote_target_id(
    st: web::Data<AppState>,
    note_id: web::Path<NoteID>,
    auth: web::ReqData<AuthedUser>,
) -> ServiceResult<impl Responder> {
    let user_id = auth.user_id_unwrap();
    let note_id = note_id.into_inner();

    // let data = st.request_data();
    delete_renote_by_id(st.conn(), st.qconn(), note_id, user_id, st.base_url()).await?;

    Ok(HttpResponse::NoContent()
        .insert_header(("hx-refresh", "true"))
        .finish())
}

#[derive(Debug, MultipartForm)]
pub struct EditNoteRequest {
    content: MpText<String>,
    #[multipart(rename = "contentType")]
    content_type: MpText<CreatableNoteContentType>,
}

#[patch("/{note_id}", wrap = "from_fn(middleware_auth_jwt_required)")]
pub async fn api_note_patch(
    st: web::Data<AppState>,
    note_id: web::Path<NoteID>,
    auth: web::ReqData<AuthedUser>,
    MultipartForm(req): MultipartForm<EditNoteRequest>,
) -> ServiceResult<impl Responder> {
    let user_id = auth.user_id_unwrap();
    let note_id = note_id.into_inner();

    let data = st.request_data();
    edit_note(
        st.conn(),
        &st.rconn(),
        st.qconn(),
        user_id,
        note_id,
        &req.content,
        req.content_type.0.into_content_type(),
        None,
        None,
        &st.my_domain(),
        st.base_url(),
        &data,
    )
    .await?;

    Ok(HttpResponse::SeeOther()
        .insert_header(("location", format!("/note/{}", note_id)))
        .finish())
}

#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct NoteUserListQuery {
    key: Option<String>,
}

pub fn create_user_list_from_models(
    iter: impl IntoIterator<Item = (SimpleUserModel, impl Into<Option<DateTime<Utc>>>)>,
) -> ServiceResult<Vec<PartsUserListData>> {
    let mut result = vec![];

    for (user, time) in iter {
        result.push(PartsUserListData {
            user: PartsUserListDataUser {
                id: user.id,
                nickname: user.nickname,
                specifier: user.specifier,
            },
            created_at: time.into(),
        });
    }

    Ok(result)
}

#[get("/{note_id}/renotes", wrap = "from_fn(middleware_auth_jwt_optional)")]
pub async fn api_get_note_renote_users(
    st: web::Data<AppState>,
    note_id: web::Path<NoteID>,
    query: web::Query<NoteUserListQuery>,
    auth: web::ReqData<AuthedUser>,
) -> ServiceResult<impl Responder> {
    let note_id = note_id.into_inner();
    let viewer_id = auth.user_id();

    let paginator = Paginator::new(
        20,
        |limit, key| {
            let conn = st.maybe_conn();
            let rconn = st.rconn();
            Box::pin(async move {
                let result =
                    get_renoted_users(&conn, &rconn, viewer_id, note_id, limit as u64, key).await?;
                let result = create_user_list_from_models(result)?;
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
            "/note/{}/renotes?{}",
            note_id,
            serde_qs::to_string(&new_query.0).unwrap()
        )
    });

    let user_list = PartsUserList {
        data: result.data,
        next_url: result.next_url,
    };

    render_template(st.template(), &Template::PartsUserList(user_list))
}

#[get("/{note_id}/likes", wrap = "from_fn(middleware_auth_jwt_optional)")]
pub async fn api_get_note_like_users(
    st: web::Data<AppState>,
    note_id: web::Path<NoteID>,
    query: web::Query<NoteUserListQuery>,
    auth: web::ReqData<AuthedUser>,
) -> ServiceResult<impl Responder> {
    let note_id = note_id.into_inner();
    let viewer_id = auth.user_id();

    let paginator = Paginator::new(
        20,
        |limit, key| {
            let conn = st.maybe_conn();
            let rconn = st.rconn();
            Box::pin(async move {
                let result =
                    get_liked_users(&conn, &rconn, viewer_id, note_id, limit as u64, key).await?;
                let result = create_user_list_from_models(result)?;
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
            "/note/{}/likes?{}",
            note_id,
            serde_qs::to_string(&new_query.0).unwrap()
        )
    });

    let user_list = PartsUserList {
        data: result.data,
        next_url: result.next_url,
    };

    render_template(st.template(), &Template::PartsUserList(user_list))
}

#[get("/{note_id}/mentions", wrap = "from_fn(middleware_auth_jwt_optional)")]
pub async fn api_get_note_mentions_users(
    st: web::Data<AppState>,
    note_id: web::Path<NoteID>,
    auth: web::ReqData<AuthedUser>,
) -> ServiceResult<impl Responder> {
    let note_id = note_id.into_inner();
    let viewer_id = auth.user_id();

    let note_details = match get_note_by_id_visibility_check(
        &st.maybe_conn(),
        &st.rconn(),
        note_id,
        viewer_id,
        false,
    )
    .await?
    {
        Some(note) => note,
        None => return create_error_simple(StatusCode::NOT_FOUND, "note not found"),
    };

    let user_list = PartsUserList {
        next_url: None,
        data: note_details
            .details
            .mentions
            .into_iter()
            .map(|m| PartsUserListData {
                user: PartsUserListDataUser {
                    id: m.id,
                    nickname: m.nickname.clone(),
                    specifier: UserSpecifier::username_and_domain_opt(m.username, m.domain)
                        .to_string(),
                },
                created_at: None,
            })
            .collect(),
    };

    render_template(st.template(), &Template::PartsUserList(user_list))
}
