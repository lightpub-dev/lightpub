use entity::sea_orm_active_enums::Visibility;
use sea_orm::Condition;
use sea_orm::prelude::*;
use serde::{Deserialize, Serialize};

use crate::services::MapToUnknown;
use crate::services::id::Identifier;
use crate::{
    ServiceResult,
    services::{
        db::MaybeTxConn,
        follow::{FollowState, is_following},
        id::{NoteID, UserID},
    },
};

use super::get::get_note_by_id;

/// ノートが閲覧者に可視かチェックする。
/// ノートが存在しない場合は false を返す。
/// viewer_id が None の場合はログインしていないユーザーを表す。
pub async fn note_visibility_check(
    conn: &MaybeTxConn,
    note_id: NoteID,
    viewer_id: Option<UserID>,
    allow_deleted: bool,
) -> ServiceResult<bool> {
    let note = get_note_by_id(conn, note_id, allow_deleted).await?;

    match note {
        None => Ok(false),
        Some(note) => {
            // check if viewer is author
            if viewer_id.is_some_and(|viewer_id| note.author.id == viewer_id) {
                return Ok(true);
            }

            let viz = note.visibility;
            match viz {
                VisibilityModel::Public | VisibilityModel::Unlisted => Ok(true),
                VisibilityModel::Private | VisibilityModel::Follower => {
                    if let Some(viewer_id) = viewer_id {
                        // check if the note mentions viewer
                        let mention = entity::note_mention::Entity::find()
                            .filter(
                                Condition::all()
                                    .add(entity::note_mention::Column::NoteId.eq(note_id.as_db()))
                                    .add(
                                        entity::note_mention::Column::TargetUserId
                                            .eq(viewer_id.as_db()),
                                    ),
                            )
                            .one(conn)
                            .await
                            .map_err_unknown()?;
                        if mention.is_some() {
                            return Ok(true);
                        }

                        // check if viewer follows author
                        if viz == VisibilityModel::Follower {
                            let follow = is_following(conn, viewer_id, note.author.id)
                                .await
                                .map(|s| s == FollowState::Yes)?;
                            if follow {
                                return Ok(true);
                            }
                        }

                        Ok(false)
                    } else {
                        Ok(false)
                    }
                }
            }
        }
    }
}

#[derive(Debug, Clone, Copy, Serialize, Deserialize, PartialEq, Eq)]
pub enum VisibilityModel {
    Public,
    Unlisted,
    Follower,
    Private,
}

impl VisibilityModel {
    pub fn from_db(v: Visibility) -> Self {
        match v {
            Visibility::Public => VisibilityModel::Public,
            Visibility::Unlisted => VisibilityModel::Unlisted,
            Visibility::Follower => VisibilityModel::Follower,
            Visibility::Private => VisibilityModel::Private,
        }
    }

    pub fn as_db(self) -> Visibility {
        match self {
            VisibilityModel::Public => Visibility::Public,
            VisibilityModel::Unlisted => Visibility::Unlisted,
            VisibilityModel::Follower => Visibility::Follower,
            VisibilityModel::Private => Visibility::Private,
        }
    }

    pub fn is_renotable(self) -> bool {
        matches!(self, VisibilityModel::Public | VisibilityModel::Unlisted)
    }

    pub fn reply_compatible(self, reply_visibility: VisibilityModel) -> bool {
        match (self, reply_visibility) {
            (
                VisibilityModel::Public | VisibilityModel::Unlisted | VisibilityModel::Follower,
                _,
            ) => true,
            (VisibilityModel::Private, VisibilityModel::Private) => true,
            _ => false,
        }
    }
}
