use chrono::{DateTime, Utc};
use nestify::nest;
use serde::{Deserialize, Serialize};
use serde_json::json;

use crate::{
    ServiceResult,
    services::{MapToUnknown, id::NoteID},
};

use super::{FTClient, FulltextID};

const NOTE_COLLECTION_NAME: &str = "notes";
const NOTE_QUERY_FIELD: &str = "content";

#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct FTNoteData {
    pub id: FulltextID<NoteID>,
    pub content: String,
    pub created_at: DateTime<Utc>,
}

impl FTNoteData {
    pub fn new(id: NoteID, content: String, created_at: DateTime<Utc>) -> Self {
        Self {
            id: FulltextID(id),
            content,
            created_at,
        }
    }
}

pub async fn setup_note_collection(client: &FTClient) -> ServiceResult<()> {
    let payload = json!({
        "name": NOTE_COLLECTION_NAME,
        "fields": [
            {"name": "content", "type": "string", "locale": client.content_locale()},
            {"name": "created_at", "type": "int64"}, // unix timestamp
        ],
    });

    client
        .client()
        .post(client.make_url("/collections"))
        .headers(client.make_headers())
        .body(payload.to_string())
        .send()
        .await
        .map_err_unknown()?;
    Ok(())
}

pub async fn push_note_index(client: &FTClient, note: &FTNoteData) -> ServiceResult<()> {
    let payload = serde_json::to_string(note).map_err_unknown()?;

    client
        .client()
        .post(client.make_url(format!(
            "/collections/{NOTE_COLLECTION_NAME}/documents?action=upsert"
        )))
        .headers(client.make_headers())
        .body(payload)
        .send()
        .await
        .map_err_unknown()?;

    Ok(())
}

pub async fn push_note_index_bulk<'a, I>(client: &FTClient, notes: I) -> ServiceResult<()>
where
    I: IntoIterator<Item = &'a FTNoteData>,
{
    let mut payload = String::new();
    for note in notes.into_iter() {
        let note_payload = serde_json::to_string(note).map_err_unknown()?;
        payload.push_str(&note_payload);
        payload.push('\n');
    }

    client
        .client()
        .post(client.make_url(format!(
            "/collections/{NOTE_COLLECTION_NAME}/documents/import?action=upsert"
        )))
        .headers(client.make_headers())
        .body(payload)
        .send()
        .await
        .map_err_unknown()?;

    Ok(())
}

pub async fn truncate_note_index(client: &FTClient) -> ServiceResult<()> {
    client
        .client()
        .delete(client.make_url(format!(
            "/collections/{NOTE_COLLECTION_NAME}/documents?truncate=true"
        )))
        .headers(client.make_headers())
        .send()
        .await
        .map_err_unknown()?;
    Ok(())
}

#[derive(Debug, Clone, Serialize)]
struct NoteSearchQuery<'a> {
    q: &'a str,
    query_by: &'a str,
}

nest! {
    #[derive(Debug, Clone, Deserialize)]*
    struct NoteSearchResponse {
        hits: Vec<struct NoteSearchResponseHits {
            document: FTNoteData,
        }>
    }
}

pub async fn ft_search_note_by_content(
    client: &FTClient,
    content_query: &str,
) -> ServiceResult<Vec<FTNoteData>> {
    let mut url = client.make_url(format!(
        "/collections/{NOTE_COLLECTION_NAME}/documents/search"
    ));
    let query = NoteSearchQuery {
        q: content_query,
        query_by: NOTE_QUERY_FIELD,
    };
    let query_string = serde_qs::to_string(&query).map_err_unknown()?;
    url.set_query(Some(&query_string));

    let response = client
        .client()
        .get(url)
        .headers(client.make_headers())
        .send()
        .await
        .map_err_unknown()?;
    let response = response
        .json::<NoteSearchResponse>()
        .await
        .map_err_unknown()?;

    Ok(response.hits.into_iter().map(|h| h.document).collect())
}
