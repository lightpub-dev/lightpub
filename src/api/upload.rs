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

use crate::middleware::strip_body;
use crate::AppState;
use actix_middleware_etag::Etag;
use actix_web::middleware::from_fn;
use actix_web::route;
/// Route handlers for /upload
use actix_web::{
    http::{
        header::{self, CacheControl, CacheDirective},
        StatusCode,
    },
    web, Either, HttpResponse, Responder,
};
use lightpub_service::services::{
    id::UploadID,
    upload::{get_upload, get_uploads_dir, GetUpload, ProxyResult},
    MapToUnknown, ServiceResult,
};
use tracing::debug;
#[route(
    "/upload/{upload_id}",
    method = "GET",
    method = "HEAD",
    wrap = "from_fn(strip_body)",
    wrap = "Etag::default()"
)]
pub async fn api_get_upload(
    st: web::Data<AppState>,
    upload_id: web::Path<UploadID>,
) -> ServiceResult<Either<impl Responder, impl Responder>> {
    let upload_id = upload_id.into_inner();
    let client = st.proxy_client();

    let upload = get_upload(&st.maybe_conn(), upload_id, client).await?;

    match upload {
        GetUpload::Proxy { res, cache_control } => match res {
            ProxyResult::Failed(status) => Ok(Either::Left({
                let mut res = HttpResponse::build(status);
                if let Some(cc) = cache_control {
                    res.insert_header((header::CACHE_CONTROL, cc.to_string()));
                }
                res.finish()
            })),
            ProxyResult::Success { res, content_type } => {
                let mut r = HttpResponse::build(StatusCode::OK);
                r.content_type(content_type);
                r.insert_header(CacheControl(vec![
                    CacheDirective::Public,
                    CacheDirective::MaxAge(UPLOAD_MAX_AGE),
                ]));
                Ok(Either::Left(r.body(res.bytes().await.map_err_unknown()?)))
            }
        },
        GetUpload::Local {
            relative_path,
            mime_type,
        } => {
            let filepath = get_uploads_dir().join(&relative_path);
            debug!("Serving local file: {:?}", filepath);
            let f = tokio::fs::read(filepath).await.map_err_unknown()?;
            Ok(Either::Right(
                HttpResponse::Ok()
                    .content_type(mime_type)
                    .insert_header(CacheControl(vec![
                        CacheDirective::Public,
                        CacheDirective::MaxAge(UPLOAD_MAX_AGE),
                        CacheDirective::Extension("immutable".to_string(), None),
                    ]))
                    .body(f),
            ))
        }
    }
}

const UPLOAD_MAX_AGE: u32 = 60 * 60 * 24 * 7;
