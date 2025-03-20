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

use std::sync::Arc;

use activitypub_federation::FEDERATION_CONTENT_TYPE;
use actix_web::{
    body::BoxBody,
    get,
    http::header::{self, ContentType, HeaderMap, TryIntoHeaderValue},
    web::Header,
    FromRequest, HttpResponse, Responder, ResponseError,
};
use derive_builder::Builder;
use handlebars::Handlebars;
use lightpub_service::services::ServiceError;
use serde::{Deserialize, Serialize};
use tracing::debug;

pub mod admin;
pub mod auth;
pub mod federation;
pub mod note;
pub mod notifications;
pub mod pagination;
pub mod search;
pub mod timeline;
pub mod trends;
pub mod upload;
pub mod user;

#[derive(Debug, Builder)]
pub struct APIResponse<T> {
    data: T,
    #[builder(default, setter(into, strip_option))]
    redirect_to: Option<String>,
    #[builder(default)]
    do_refresh: bool,
    #[builder(default, setter(into, strip_option))]
    trigger_event: Option<String>,
}

impl<T: Serialize> Responder for APIResponse<T> {
    type Body = BoxBody;

    fn respond_to(self, _req: &actix_web::HttpRequest) -> actix_web::HttpResponse<Self::Body> {
        let mut res = HttpResponse::Ok();
        res.insert_header(("content-type", "application/json"));

        if let Some(redirect_to) = self.redirect_to.as_ref() {
            res.insert_header(("hx-redirect", redirect_to.as_str()));
        }

        if let Some(trigger_event) = self.trigger_event.as_ref() {
            res.insert_header(("hx-trigger", trigger_event.as_str()));
        }

        if self.do_refresh {
            res.insert_header(("hx-refresh", "true"));
        }

        res.json(self.data)
    }
}

#[derive(Debug, Builder)]
pub struct HtmxResponse<'a, T> {
    data: T,
    template: Arc<Handlebars<'a>>,
    template_name: &'a str,
}

impl<'a, T: Serialize> Responder for HtmxResponse<'a, T> {
    type Body = BoxBody;

    fn respond_to(self, _req: &actix_web::HttpRequest) -> HttpResponse<Self::Body> {
        let rendered = self.template.render(self.template_name, &self.data);

        match rendered {
            Ok(rendered) => HttpResponse::Ok()
                .insert_header((
                    header::CONTENT_TYPE,
                    ContentType::html().try_into_value().unwrap(),
                ))
                .body(rendered),

            Err(e) => ServiceError::unknown(e).error_response(),
        }
    }
}
#[derive(Debug, Clone, Copy, PartialEq, Eq)]
pub struct FormBool(pub bool);

impl<'de> Deserialize<'de> for FormBool {
    fn deserialize<D>(deserializer: D) -> Result<Self, D::Error>
    where
        D: serde::Deserializer<'de>,
    {
        let s: Option<&str> = Deserialize::deserialize(deserializer)?;
        Ok(FormBool(s.is_some_and(|s| s == "on")))
    }
}

impl FormBool {
    pub fn value(&self) -> bool {
        self.0
    }
}

#[derive(Debug, Clone, Copy, PartialEq, Eq, Hash)]
pub enum RequestType {
    HTML,
    APUB,
    HTMX,
}

impl FromRequest for RequestType {
    type Error = actix_web::Error;
    type Future = std::future::Ready<Result<Self, Self::Error>>;

    fn from_request(
        req: &actix_web::HttpRequest,
        _payload: &mut actix_web::dev::Payload,
    ) -> Self::Future {
        let headers = req.headers();
        let request_type = judge_request_type(headers);
        debug!("Request type: {:?}", request_type);
        std::future::ready(Ok(request_type))
    }
}

const JSON_LD_CONTENT_TYPE: &str = "application/ld+json";

pub fn judge_request_type(headers: &HeaderMap) -> RequestType {
    if headers.contains_key("hx-request") {
        RequestType::HTMX
    } else if headers.get("accept").is_some_and(|h| {
        h.to_str()
            .is_ok_and(|s| s.contains(FEDERATION_CONTENT_TYPE) || s.contains(JSON_LD_CONTENT_TYPE))
    }) {
        RequestType::APUB
    } else {
        RequestType::HTML
    }
}

pub fn accept_apub(accept: &Header<header::Accept>) -> bool {
    accept
        .iter()
        .any(|v| v.item.to_string() == FEDERATION_CONTENT_TYPE)
}

pub fn render_apub(data: impl Serialize) -> HttpResponse {
    HttpResponse::Ok()
        .insert_header((header::CONTENT_TYPE, FEDERATION_CONTENT_TYPE))
        .json(data)
}

#[derive(Debug, Clone, Serialize, Deserialize)]
#[serde(rename_all = "camelCase")]
pub struct WithAuthed<T> {
    authed: bool,
    #[serde(flatten)]
    data: T,
}

impl<T> WithAuthed<T> {
    pub fn with(authed: bool, data: T) -> Self {
        Self { authed, data }
    }
}

// service worker needs to be served in the root path
#[get("/sw.js")]
pub async fn serve_sw_js() -> impl Responder {
    actix_files::NamedFile::open("static/js/sw.js")
}

#[cfg(test)]
mod test {
    use actix_web::http::header::HeaderMap;

    use crate::api::{judge_request_type, RequestType};

    #[test]
    fn test_judge_request_type() {
        use actix_web::http::header::{HeaderName, HeaderValue};

        let mut headers = HeaderMap::new();
        headers.insert(
            HeaderName::from_static("hx-request"),
            HeaderValue::from_static("true"),
        );
        assert_eq!(judge_request_type(&headers), RequestType::HTMX);

        headers.clear();
        headers.insert(
            HeaderName::from_static("accept"),
            HeaderValue::from_static("text/html,application/activity+json"),
        );
        assert_eq!(judge_request_type(&headers), RequestType::APUB);

        headers.clear();
        headers.insert(
            HeaderName::from_static("accept"),
            HeaderValue::from_static("text/html"),
        );
        assert_eq!(judge_request_type(&headers), RequestType::HTML);
    }
}
