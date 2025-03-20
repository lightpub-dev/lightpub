use crate::{
    ServiceResult,
    services::{
        db::MaybeTxConn,
        fulltext::{FTClient, note::ft_search_note_by_content},
        id::UserID,
        kv::KVObject,
        note::{DetailedNoteModel, get_note_by_id_visibility_check},
    },
};

pub async fn search_note_by_content(
    conn: &MaybeTxConn,
    rconn: &KVObject,
    ft: &FTClient,
    content_query: &str,
    viewer_id: Option<UserID>,
) -> ServiceResult<Vec<DetailedNoteModel>> {
    let ft_notes = ft_search_note_by_content(ft, content_query).await?;

    let mut notes = Vec::new();
    for ft_note in ft_notes {
        let note =
            get_note_by_id_visibility_check(conn, rconn, ft_note.id.into_inner(), viewer_id, false)
                .await?;
        if let Some(note) = note {
            notes.push(note);
        }
    }

    Ok(notes)
}
