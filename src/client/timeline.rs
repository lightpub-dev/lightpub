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

use crate::client::middleware_auth_jwt_optional;
use crate::{
    api::auth::AuthedUser,
    template::{render_template, Template, Timeline},
    AppState,
};
use actix_web::middleware::from_fn;
use actix_web::{get, web, HttpResponse, Responder};
use lightpub_service::ServiceResult;
use serde::{Deserialize, Serialize};

#[derive(Debug, Clone, Deserialize, Serialize)]
pub struct TimelineQuery {
    public: Option<bool>,
}

#[get("/timeline", wrap = "from_fn(middleware_auth_jwt_optional)")]
pub async fn client_timeline(
    st: web::Data<AppState>,
    query: web::Query<TimelineQuery>,
    auth: web::ReqData<AuthedUser>,
) -> ServiceResult<impl Responder> {
    if !auth.is_authed() && !query.public.unwrap_or(false) {
        return Ok(HttpResponse::Found()
            .insert_header(("location", "/client/login"))
            .finish());
    }

    let temp = &Template::Timeline(Timeline {
        authed: auth.is_authed().into(),
        timeline_url: format!("/timeline?{}", serde_qs::to_string(&query.0).unwrap()),
    });

    render_template(st.template(), &temp)
}
