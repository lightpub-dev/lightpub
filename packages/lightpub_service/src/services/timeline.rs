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

use crate::services::{MapToUnknown, id::Identifier};
use chrono::{DateTime, Utc};
use sea_orm::query::*;
use serde::{Deserialize, Serialize};

use super::{
    ServiceResult,
    db::MaybeTxConn,
    id::{NoteID, UserID},
};

pub async fn get_timeline_note_ids(
    conn: &MaybeTxConn,
    viewer_id: Option<UserID>,
    include_public: bool,
    limit: u64,
    before_date: Option<DateTime<Utc>>,
) -> ServiceResult<Vec<NoteID>> {
    get_note_ids_generalized(
        conn,
        viewer_id,
        true,
        include_public,
        false,
        None,
        limit,
        before_date,
    )
    .await
}

pub async fn get_note_reply_ids(
    conn: &MaybeTxConn,
    viewer_id: Option<UserID>,
    target_note_id: NoteID,
    limit: u64,
    before_date: Option<DateTime<Utc>>,
) -> ServiceResult<Vec<NoteID>> {
    get_note_ids_generalized(
        conn,
        viewer_id,
        true,
        true,
        true,
        Some(target_note_id),
        limit,
        before_date,
    )
    .await
}

async fn get_note_ids_generalized(
    conn: &MaybeTxConn,
    viewer_id: Option<UserID>,
    include_self: bool,
    include_public: bool,
    include_unlisted: bool,
    limit_reply_to_id: Option<NoteID>,
    limit: u64,
    before_date: Option<DateTime<Utc>>,
) -> ServiceResult<Vec<NoteID>> {
    let ids = conn
        .query_all(Statement::from_sql_and_values(
            conn.get_database_backend(),
            r#"CALL get_note_ids_generalized(?,?,?,?,?,?,?)"#,
            [
                viewer_id.map(|a| a.as_db()).into(),
                include_self.into(),
                include_public.into(),
                include_unlisted.into(),
                limit_reply_to_id.map(|a| a.as_db()).into(),
                limit.into(),
                before_date.map(|d| d.naive_utc()).into(),
            ],
        ))
        .await
        .map_err_unknown()?;

    Ok(ids
        .into_iter()
        // BUG: sqlx does not support try_get by column name when using stored procedures
        // https://github.com/launchbadge/sqlx/issues/1742
        .map(|n| NoteID::from_db_trusted(n.try_get_by(0).unwrap()))
        .collect())
}

#[derive(Debug, Clone, Serialize, Deserialize)]
#[serde(rename_all = "camelCase")]
pub struct NoteTrends {
    pub data: Vec<TrendEntry>,
}

#[derive(Debug, Clone, Serialize, Deserialize)]
#[serde(rename_all = "camelCase")]
pub struct TrendEntry {
    pub hashtag: String,
    pub count: u64,
    pub url: String,
}

pub async fn get_trending_hashtags(conn: &MaybeTxConn) -> ServiceResult<NoteTrends> {
    let db = conn
        .query_all(Statement::from_sql_and_values(
            conn.get_database_backend(),
            r#"SELECT * FROM trending_tags LIMIT 5"#,
            [5.into()],
        ))
        .await
        .map_err_unknown()?;

    let mut trends = vec![];
    for d in db {
        let hashtag: String = d.try_get("", "name").map_err_unknown()?;
        let count: i64 = d.try_get("", "note_count").map_err_unknown()?;
        let url = format!(
            "/timeline?tag={}",
            percent_encoding::percent_encode(hashtag.as_bytes(), percent_encoding::CONTROLS,)
        );
        trends.push(TrendEntry {
            hashtag,
            count: count as u64,
            url,
        });
    }

    Ok(NoteTrends { data: trends })
}
