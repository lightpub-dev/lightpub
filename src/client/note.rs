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

use crate::{
    api::{auth::AuthedUser, note::create_parts_note_from_model},
    template::{render_template, NoteDetails, NoteOg, Template, UserList},
    AppState,
};
use actix_web::{get, http::StatusCode, web, Responder};
use lightpub_service::services::{
    create_error_simple, id::NoteID, note::get_note_by_id_visibility_check, ServiceResult,
};

#[get("/note/{note_id}", wrap = "from_fn(middleware_auth_jwt_optional)")]
pub async fn client_get_note(
    st: web::Data<AppState>,
    note_id: web::Path<(NoteID,)>,
    auth: web::ReqData<AuthedUser>,
) -> ServiceResult<impl Responder> {
    let note_id = note_id.into_inner().0;
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
            let note = create_parts_note_from_model(&st, &note, auth.user_id(), None).await?;
            let temp = Template::NoteDetails(NoteDetails {
                authed: auth.is_authed().into(),
                og: NoteOg {
                    base_url: st.base_url().clone(),
                    url: st
                        .base_url()
                        .join(format!("/client/note/{}", note.note.id).as_str())
                        .unwrap(),
                },
                note: note.note,
            });
            render_template(st.template(), &temp)
        }
    }
}
use crate::client::middleware_auth_jwt_optional;
use actix_web::middleware::from_fn;
#[get(
    "/note/{note_id}/renotes",
    wrap = "from_fn(middleware_auth_jwt_optional)"
)]
pub async fn client_note_renotes_list(
    st: web::Data<AppState>,
    note_id: web::Path<NoteID>,
) -> ServiceResult<impl Responder> {
    let temp = Template::UserList(UserList {
        url: format!("/note/{}/renotes", note_id),
        title: "リノート一覧".to_owned(),
    });
    render_template(st.template(), &temp)
}

#[get(
    "/note/{note_id}/likes",
    wrap = "from_fn(middleware_auth_jwt_optional)"
)]
pub async fn client_note_liked_list(
    st: web::Data<AppState>,
    note_id: web::Path<NoteID>,
) -> ServiceResult<impl Responder> {
    let temp = Template::UserList(UserList {
        url: format!("/note/{}/likes", note_id),
        title: "お気に入り一覧".to_owned(),
    });
    render_template(st.template(), &temp)
}

#[get(
    "/note/{note_id}/mentions",
    wrap = "from_fn(middleware_auth_jwt_optional)"
)]
pub async fn client_note_mentions_list(
    st: web::Data<AppState>,
    note_id: web::Path<NoteID>,
) -> ServiceResult<impl Responder> {
    let temp = Template::UserList(UserList {
        url: format!("/note/{}/mentions", note_id),
        title: "メンション一覧".to_owned(),
    });
    render_template(st.template(), &temp)
}
