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

/// Route handlers for /client
use actix_web::{
    body::{BoxBody, MessageBody},
    dev::{ServiceRequest, ServiceResponse},
    http::header::{self},
    middleware::Next,
    web, HttpResponse,
};
use expected_error::StatusCode;
use expected_error_derive::ExpectedError;
use thiserror::Error;

use lightpub_service::services::{user::UserSpecifier, ServiceError, ServiceResult};

use crate::api::auth::{middleware_auth_jwt_optional, AuthedUser};

pub mod note;
pub mod notification;
pub mod search;
pub mod template;
pub mod timeline;
pub mod user;

#[derive(Debug, Error, ExpectedError)]
pub enum UserSpecParseError {
    #[error("invalid user specifier")]
    #[ee(status(StatusCode::BAD_REQUEST))]
    Invalid,
}

pub fn parse_user_spec(spec: &str, my_domain: &str) -> ServiceResult<UserSpecifier> {
    UserSpecifier::from_str(spec, my_domain).ok_or(ServiceError::known(UserSpecParseError::Invalid))
}

/// Middleware that redirects to login page if user is not authenticated
/// Must be called after `middleware_auth_jwt_optional`
pub async fn middleware_redirect_login(
    auth_user: web::ReqData<AuthedUser>,
    req: ServiceRequest,
    next: Next<BoxBody>,
) -> Result<ServiceResponse<impl MessageBody>, actix_web::Error> {
    if auth_user.is_authed() {
        next.call(req).await
    } else {
        let url = "/client/login";
        Ok(req.into_response(
            HttpResponse::Found()
                .insert_header((header::LOCATION, url))
                .finish(),
        ))
    }
}
