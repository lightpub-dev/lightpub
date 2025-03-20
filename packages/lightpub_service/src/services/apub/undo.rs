use activitypub_federation::{
    config::Data, fetch::object_id::ObjectId, kinds::activity::UndoType, traits::ActivityHandler,
};
use async_trait::async_trait;
use derive_more::From;
use serde::{Deserialize, Serialize};
use url::Url;

use crate::{
    MyFederationData,
    services::{
        FederationServiceError,
        follow::unfollow_user,
        note::{delete_renote_by_id, note_like_remove},
        user::UserWithApubModel,
    },
};

use super::{
    AnnounceActivity, AnnounceableObject, LikeActivity, LikeableObject, actor_check,
    follow::FollowActivity,
};

#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct UndoActivity {
    pub id: Url,
    #[serde(rename = "type")]
    pub kind: UndoType,
    pub actor: ObjectId<UserWithApubModel>,
    pub object: UndoableObject,
}

impl UndoActivity {
    pub fn new(actor: Url, object: impl Into<UndoableObject>) -> Self {
        let object = object.into();
        let undo_id = Url::parse(format!("{}/undo", object.id()).as_str()).unwrap();
        Self {
            id: undo_id,
            kind: UndoType::Undo,
            actor: ObjectId::from(actor),
            object,
        }
    }
}

#[derive(Debug, Clone, Serialize, Deserialize, From)]
#[serde(untagged)]
pub enum UndoableObject {
    Follow(FollowActivity),
    Like(LikeActivity),
    Announce(AnnounceActivity),
}

impl UndoableObject {
    pub fn id(&self) -> &Url {
        match self {
            UndoableObject::Follow(follow) => follow.id(),
            UndoableObject::Like(like) => like.id(),
            UndoableObject::Announce(announce) => announce.id(),
        }
    }

    pub fn actor(&self) -> &ObjectId<UserWithApubModel> {
        match self {
            UndoableObject::Follow(follow) => &follow.actor,
            UndoableObject::Like(like) => &like.actor,
            UndoableObject::Announce(announce) => &announce.actor,
        }
    }
}

#[async_trait]
impl ActivityHandler for UndoActivity {
    type DataType = MyFederationData;
    type Error = FederationServiceError;

    fn id(&self) -> &Url {
        &self.id
    }

    fn actor(&self) -> &Url {
        self.actor.inner()
    }

    async fn verify(&self, _data: &Data<Self::DataType>) -> Result<(), Self::Error> {
        // actor check
        match &self.object {
            UndoableObject::Follow(follow) => actor_check(self.actor(), follow.actor())?, // actor == follower
            UndoableObject::Like(like) => actor_check(self.actor(), like.actor())?, // actor == liker
            UndoableObject::Announce(an) => actor_check(self.actor(), an.actor())?, // actor = announcer
        }

        Ok(())
    }

    async fn receive(self, data: &Data<Self::DataType>) -> Result<(), Self::Error> {
        match self.object {
            UndoableObject::Follow(follow) => {
                let actor = follow.actor.dereference(data).await?;
                let followed = follow.object.dereference(data).await?;
                unfollow_user(
                    data.conn(),
                    &data.rconn(),
                    data.qconn(),
                    actor.basic.id,
                    followed.basic.id,
                    data.base_url(),
                )
                .await?;
            }
            UndoableObject::Like(like) => {
                let actor = like.actor.dereference(data).await?;
                match like.object {
                    LikeableObject::Note(note) => {
                        let note = note.dereference(data).await?;
                        note_like_remove(
                            data.conn(),
                            data.qconn(),
                            actor.basic.id,
                            note.basic.id,
                            false,
                            data.base_url(),
                        )
                        .await?;
                    }
                }
            }
            UndoableObject::Announce(an) => {
                let actor = an.actor.dereference(data).await?;
                match an.object {
                    AnnounceableObject::Note(renote_target_id) => {
                        let renote_target = renote_target_id.dereference(data).await?;
                        delete_renote_by_id(
                            data.conn(),
                            data.qconn(),
                            renote_target.basic.id,
                            actor.basic.id,
                            data.base_url(),
                        )
                        .await?;
                    }
                }
            }
        }

        Ok(())
    }
}
