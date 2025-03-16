use crate::{
    ServiceResult,
    services::{id::NoteID, kv::KVObject},
};

fn make_note_kv_key(note_id: NoteID, attr: Option<&str>) -> String {
    match attr {
        None => format!("note:{}", note_id),
        Some(a) => format!("note:{}:{}", a, note_id),
    }
}

pub async fn invalidate_note_basic_cache(rconn: &KVObject, note_id: NoteID) -> ServiceResult<()> {
    rconn.delete(make_note_kv_key(note_id, None)).await?;

    Ok(())
}
