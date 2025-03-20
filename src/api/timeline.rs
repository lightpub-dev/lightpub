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

/// Route handlers for /timeline
use crate::{
    api::{note::create_parts_note_from_model, pagination::Paginator},
    template::{render_template, PartsNotes, Template},
    AppState,
};
use actix_web::{get, http::StatusCode, web, Responder};
use lightpub_service::services::{create_error_simple, note::get_timeline_notes, ServiceResult};
use percent_encoding::CONTROLS;
use serde::{Deserialize, Serialize};

use super::{auth::AuthedUser, pagination::URLBasedPaginatedResponse};

#[derive(Debug, Serialize, Deserialize)]
pub struct TimelineNextKey {
    pub bd: Option<chrono::DateTime<chrono::Utc>>, // before_date
}

#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct GetTimelineQuery {
    pub key: Option<String>,
    pub public: Option<bool>,
}
use crate::api::auth::middleware_auth_jwt_optional;
use actix_web::middleware::from_fn;
#[get("/timeline", wrap = "from_fn(middleware_auth_jwt_optional)")]
pub async fn api_get_timeline(
    st: web::Data<AppState>,
    auth: web::ReqData<AuthedUser>,
    query: web::Query<GetTimelineQuery>,
) -> ServiceResult<impl Responder> {
    let user_id = auth.user_id();
    let include_public_query = query.public;
    let include_public = include_public_query.unwrap_or(false);

    if !include_public && user_id.is_none() {
        return create_error_simple(
            StatusCode::UNAUTHORIZED,
            "you must be logged in to view your personal timeline",
        );
    }

    let paginator = Paginator::new(
        20,
        |limit: usize, key: Option<TimelineNextKey>| {
            let conn = st.maybe_conn();
            let rconn = st.rconn().clone();
            let st = st.clone();
            Box::pin(async move {
                let notes = get_timeline_notes(
                    &conn,
                    &rconn,
                    user_id,
                    include_public,
                    limit as u64,
                    key.as_ref().map(|k| k.bd).flatten(),
                )
                .await?;

                let mut note_vec = vec![];
                for n in notes {
                    note_vec.push(create_parts_note_from_model(&st, &n, user_id, None).await?);
                }
                Ok(note_vec)
            })
        },
        |n| TimelineNextKey {
            bd: Some(n.note.created_at),
        },
    );

    let response = paginator
        .fetch_page(query.key.as_ref().map(|s| s.to_owned()))
        .await?;
    let paginated = URLBasedPaginatedResponse::from_paginated(response, |k| {
        let new_query = GetTimelineQuery {
            key: Some(k),
            public: include_public_query,
        };
        let qs = serde_qs::to_string(&new_query).unwrap();
        format!(
            "/timeline?{}",
            percent_encoding::utf8_percent_encode(&qs, CONTROLS)
        )
    });

    let value = PartsNotes {
        authed: auth.is_authed().into(),
        next_url: paginated.next_url,
        data: paginated.data,
    };

    render_template(st.template(), &Template::PartsNotes(value))
}
