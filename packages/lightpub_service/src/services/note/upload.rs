use std::str::FromStr;

use crate::services::{
    MapToUnknown, ServiceResult,
    db::MaybeTxConn,
    id::{Identifier, NoteID, UploadID},
};
use derive_getters::Getters;
use sea_orm::{ColumnTrait, EntityTrait, QueryFilter};
use serde::{Deserialize, Serialize};
use url::Url;

fn convert_to_apub(model: NoteUploadModel, base_url: &Url) -> NoteUploadApubModel {
    let kind = "Image".to_string(); // TODO: infer from mime type
    let mime_type = model.mime_type;
    let url = match model.data {
        NoteUploadModelData::File(upload_id) => {
            base_url.join(&format!("/upload/{upload_id}")).unwrap()
        }
        NoteUploadModelData::Remote(_, url) => url,
    };

    NoteUploadApubModel {
        kind,
        mime_type,
        url,
    }
}

pub async fn get_note_uploads_apub(
    conn: &MaybeTxConn,
    note_id: NoteID,
    base_url: &Url,
) -> ServiceResult<Vec<NoteUploadApubModel>> {
    let uploads = get_note_uploads(conn, note_id).await?;
    Ok(uploads
        .into_iter()
        .map(|u| convert_to_apub(u, base_url))
        .collect())
}

pub async fn get_note_uploads(
    conn: &MaybeTxConn,
    note_id: NoteID,
) -> ServiceResult<Vec<NoteUploadModel>> {
    let uploads = entity::upload::Entity::find()
        .find_also_related(entity::note_upload::Entity)
        .filter(entity::note_upload::Column::NoteId.eq(note_id.as_db()))
        .all(conn)
        .await
        .map_err_unknown()?;

    let mut result = vec![];
    for (upload, _) in uploads {
        let upload_id = UploadID::from_db_trusted(upload.id);
        result.push(NoteUploadModel {
            mime_type: upload.mime_type,
            data: match (upload.filename, upload.url) {
                (Some(_), None) => NoteUploadModelData::File(upload_id),
                (None, Some(url)) => {
                    NoteUploadModelData::Remote(upload_id, Url::from_str(&url).expect("bad url"))
                }
                _ => unreachable!("invalid upload"),
            },
        })
    }

    Ok(result)
}

#[derive(Debug, Clone, Getters)]
pub struct NoteUploadApubModel {
    pub(crate) kind: String,
    pub(crate) mime_type: String,
    pub(crate) url: Url,
}

#[derive(Debug, Clone, Getters, Serialize, Deserialize)]
#[serde(rename_all = "camelCase")]
pub struct NoteUploadModel {
    mime_type: String,
    data: NoteUploadModelData,
}

#[derive(Debug, Clone, Serialize, Deserialize)]
#[serde(rename_all = "camelCase")]
pub enum NoteUploadModelData {
    File(UploadID),
    Remote(UploadID, Url),
}

impl NoteUploadModelData {
    pub fn upload_id(&self) -> UploadID {
        match self {
            NoteUploadModelData::File(upload_id) => upload_id.clone(),
            NoteUploadModelData::Remote(upload_id, _) => upload_id.clone(),
        }
    }
}
