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
use crate::client::middleware_redirect_login;
use crate::{
    template::{render_template, Template},
    AppState,
};
use actix_web::middleware::from_fn;
use actix_web::{get, web, Responder};
use lightpub_service::ServiceResult;
#[get(
    "/notification",
    wrap = "from_fn(middleware_redirect_login)",
    wrap = "from_fn(middleware_auth_jwt_optional)"
)]
pub async fn client_notification_get(st: web::Data<AppState>) -> ServiceResult<impl Responder> {
    render_template(st.template(), &Template::Notification(()))
}
