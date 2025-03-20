use activitypub_federation::{
    config::Data, fetch::object_id::ObjectId, kinds::activity::AnnounceType,
    traits::ActivityHandler,
};
use async_trait::async_trait;
use chrono::{DateTime, Utc};
use serde::{Deserialize, Serialize};
use url::Url;

use crate::{
    MyFederationData,
    services::{
        FederationServiceError,
        note::{NoteWithApubModel, create_renote},
        user::UserWithApubModel,
    },
};

#[derive(Debug, Clone, Serialize, Deserialize)]
#[serde(rename_all = "camelCase")]
pub struct AnnounceActivity {
    pub id: Url,
    #[serde(rename = "type")]
    pub kind: AnnounceType,
    pub actor: ObjectId<UserWithApubModel>,
    #[serde(default)]
    pub to: Vec<ObjectId<UserWithApubModel>>,
    #[serde(default)]
    pub cc: Vec<ObjectId<UserWithApubModel>>,
    pub object: AnnounceableObject,
    pub published: DateTime<Utc>,
}

#[derive(Debug, Clone, Serialize, Deserialize)]
#[serde(untagged)]
pub enum AnnounceableObject {
    Note(ObjectId<NoteWithApubModel>),
}

impl AnnounceActivity {
    pub fn from_note(
        target_note: ObjectId<NoteWithApubModel>,
        renote_id: Url,
        actor: ObjectId<UserWithApubModel>,
        to: Vec<ObjectId<UserWithApubModel>>,
        cc: Vec<ObjectId<UserWithApubModel>>,
        published: DateTime<Utc>,
    ) -> Self {
        Self {
            id: renote_id,
            kind: AnnounceType::Announce,
            actor,
            to,
            cc,
            object: AnnounceableObject::Note(target_note),
            published,
        }
    }
}

#[async_trait]
impl ActivityHandler for AnnounceActivity {
    type DataType = MyFederationData;
    type Error = FederationServiceError;

    fn id(&self) -> &Url {
        &self.id
    }

    fn actor(&self) -> &Url {
        self.actor.inner()
    }

    async fn verify(&self, _data: &Data<Self::DataType>) -> Result<(), Self::Error> {
        Ok(())
    }

    async fn receive(self, data: &Data<Self::DataType>) -> Result<(), Self::Error> {
        match self.object {
            AnnounceableObject::Note(note) => {
                // save renote
                let target_note = note.dereference(data).await?;
                let actor = self.actor.dereference(data).await?;
                create_renote(
                    data.conn(),
                    &data.rconn(),
                    data.qconn(),
                    data.wp(),
                    actor.basic.id,
                    target_note.basic.id,
                    target_note.basic.visibility,
                    data.base_url(),
                )
                .await?;
            }
        }

        Ok(())
    }
}
