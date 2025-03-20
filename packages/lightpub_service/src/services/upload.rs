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

use std::{
    path::{Path, PathBuf},
    str::FromStr,
    sync::Once,
};

use crate::{impl_into_known, services::ServiceError};
use expected_error_derive::ExpectedError;
use url::Url;

use super::{
    MapToUnknown, ServiceResult, create_error_simple,
    db::MaybeTxConn,
    id::{Identifier, UploadID},
};
use actix_web::http::StatusCode;
use infer::Type;
use rexiv2::Metadata;
use sea_orm::Set;
use sea_orm::{ActiveModelTrait, EntityTrait};
use tempfile::NamedTempFile;
use thiserror::Error;
use tracing::error;

static START: Once = Once::new();

#[derive(Debug, Clone, Error, ExpectedError)]
pub enum UploadError {
    #[error("exif error")]
    #[ee(status(StatusCode::BAD_REQUEST))]
    ExifError,
}

pub fn save_upload_file(tmp: NamedTempFile) -> ServiceResult<(UploadID, PathBuf, Type)> {
    let upload_id = UploadID::new_random();
    let ext = {
        let mime = infer::get_from_path(tmp.path()).map_err_unknown()?;
        mime.map(|m| format!(".{}", m.extension()))
            .unwrap_or_else(|| "".to_string())
    };
    let filename = PathBuf::from_str(&format!("{}{}", upload_id.to_string(), ext)).unwrap();

    let upload_path = get_uploads_dir().join(&filename);
    // copy tmp to upload_path
    std::fs::copy(tmp.path(), &upload_path).map_err_unknown()?;
    std::mem::drop(tmp);

    // check if it's an image
    let kind = infer::get_from_path(&upload_path).map_err_unknown()?;
    match kind {
        None => create_error_simple(StatusCode::BAD_REQUEST, "invalid file type"),
        Some(ty) => {
            if ty.mime_type().starts_with("image/") {
                remove_exif(&upload_path)?;
                return Ok((upload_id.clone(), filename, ty));
            }

            create_error_simple(StatusCode::BAD_REQUEST, "invalid file type")
        }
    }
}

fn remove_exif(upload_path: &Path) -> ServiceResult<()> {
    START.call_once(|| {
        rexiv2::initialize().expect("unable to initialize rexiv2");
    });

    let meta = Metadata::new_from_path(upload_path).map_err(|e| {
        error!("failed to read exif: {}", e);
        ServiceError::known(UploadError::ExifError)
    })?;

    meta.delete_gps_info();

    meta.save_to_file(upload_path).map_err_unknown()?;

    Ok(())
}

pub async fn save_upload_file_info(
    tx: &MaybeTxConn,
    upload_id: &UploadID,
    filename: &Path,
    mime_type: &str,
) -> ServiceResult<()> {
    let model = entity::upload::ActiveModel {
        id: Set(upload_id.as_db()),
        filename: Set(Some(filename.to_str().expect("bad filename").to_string())),
        mime_type: Set(mime_type.to_string()),
        ..Default::default()
    };
    model.insert(tx).await.map_err_unknown()?;
    Ok(())
}

pub fn get_uploads_dir() -> PathBuf {
    let upload_dir = std::env::var("UPLOAD_DIR").unwrap_or_else(|_| "./uploads".to_string());
    PathBuf::from(upload_dir)
}

#[derive(Debug)]
pub enum GetUpload {
    Local {
        relative_path: String,
        mime_type: String,
    },
    Proxy {
        cache_control: Option<String>,
        res: ProxyResult,
    },
}

#[derive(Debug)]
pub enum ProxyResult {
    Success {
        res: reqwest::Response,
        content_type: String,
    },
    Failed(StatusCode),
}

#[derive(Debug, Clone, Error, ExpectedError)]
pub enum GetUploadError {
    #[error("upload not found")]
    #[ee(status(StatusCode::NOT_FOUND))]
    NotFound,
    #[error("invalid remote")]
    #[ee(status(StatusCode::BAD_REQUEST))]
    InvalidRemote,
}

pub async fn get_upload(
    conn: &MaybeTxConn,
    upload_id: UploadID,
    client: &reqwest_middleware::ClientWithMiddleware,
) -> ServiceResult<GetUpload> {
    let upload = entity::upload::Entity::find_by_id(upload_id.as_db())
        .one(conn)
        .await
        .map_err_unknown()?;

    match upload {
        None => Err(ServiceError::known(GetUploadError::NotFound)),
        Some(upload) => {
            if let Some(filename) = upload.filename {
                Ok(GetUpload::Local {
                    relative_path: filename,
                    mime_type: upload.mime_type,
                })
            } else if let Some(url) = &upload.url {
                let expected_content_type = &upload.mime_type;

                let res = client.get(url).send().await.map_err(|e| {
                    error!("failed to fetch upload: {}", e);
                    ServiceError::unknown(e)
                })?;

                let cache_control = match res.headers().get("cahce-control") {
                    None => None,
                    Some(c) => {
                        let c = c
                            .to_str()
                            .map_err(|_| ServiceError::known(GetUploadError::InvalidRemote))?;
                        Some(c.to_string())
                    }
                };

                if res.status() != reqwest::StatusCode::OK {
                    return Ok(GetUpload::Proxy {
                        cache_control: cache_control,
                        res: ProxyResult::Failed(
                            StatusCode::from_u16(res.status().as_u16()).unwrap(),
                        ),
                    });
                }

                // content type check
                let content_type = match res.headers().get("content-type") {
                    None => return Err(ServiceError::known(GetUploadError::InvalidRemote)),
                    Some(content_type) => {
                        if content_type != expected_content_type {
                            return Err(ServiceError::known(GetUploadError::InvalidRemote));
                        }
                        content_type
                            .to_str()
                            .map_err(|_| ServiceError::known(GetUploadError::InvalidRemote))?
                            .to_string()
                    }
                };

                Ok(GetUpload::Proxy {
                    cache_control,
                    res: ProxyResult::Success { res, content_type },
                })
            } else {
                unreachable!("invalid upload")
            }
        }
    }
}

pub async fn register_remote_upload(
    conn: &MaybeTxConn,
    url: &Url,
    client: &reqwest_middleware::ClientWithMiddleware,
) -> ServiceResult<UploadID> {
    let mime_type = check_remote_mime_type(url.as_str(), client).await?;
    let upload_id = UploadID::new_random();
    let model = entity::upload::ActiveModel {
        id: Set(upload_id.as_db()),
        url: Set(Some(url.to_string())),
        mime_type: Set(mime_type),
        ..Default::default()
    };
    model.insert(conn).await.map_err_unknown()?;
    Ok(upload_id)
}

async fn check_remote_mime_type(
    url: &str,
    client: &reqwest_middleware::ClientWithMiddleware,
) -> ServiceResult<String> {
    // Send a HEAD request to the URL
    let response = client
        .head(url)
        .send()
        .await
        .map_err(|_| MimeTypeError::RequestFailed)?;

    // Get the content type from headers
    if let Some(content_type) = response.headers().get("content-type") {
        let mime_type = content_type
            .to_str()
            .map_err(|_| MimeTypeError::HeaderParseFailed)?;
        return Ok(mime_type.to_string());
    }

    // If headers don't provide content type, return an error
    Err(MimeTypeError::NoContentType.into())
}

#[derive(Error, Debug, ExpectedError)]
pub enum MimeTypeError {
    #[error("Failed to send request to the URL")]
    #[ee(status(StatusCode::BAD_GATEWAY))]
    RequestFailed,

    #[error("Failed to parse content-type header")]
    #[ee(status(StatusCode::BAD_GATEWAY))]
    HeaderParseFailed,

    #[error("No content-type header found")]
    #[ee(status(StatusCode::BAD_REQUEST))]
    NoContentType,
}

impl_into_known!(MimeTypeError);
