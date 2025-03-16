use activitypub_federation::{
    config::Data,
    fetch::object_id::ObjectId,
    kinds::activity::CreateType,
    traits::{ActivityHandler, Object},
};
use async_trait::async_trait;
use serde::{Deserialize, Serialize};
use url::Url;

use crate::{
    MyFederationData,
    services::{
        FederationServiceError,
        note::{ApubNoteModel, NoteWithApubModel},
        user::UserWithApubModel,
    },
};

use super::actor_check;

#[derive(Debug, Clone, Serialize, Deserialize)]
#[serde(rename_all = "camelCase")]
pub struct CreateActivity {
    pub id: Url,
    #[serde(rename = "type")]
    pub kind: CreateType,
    pub actor: ObjectId<UserWithApubModel>,
    #[serde(default)]
    pub to: Vec<Url>,
    #[serde(default)]
    pub cc: Vec<Url>,
    pub object: CreatableObject,
}

#[derive(Debug, Clone, Serialize, Deserialize)]
#[serde(untagged)]
pub enum CreatableObject {
    Note(ApubNoteModel),
}

impl CreateActivity {
    pub fn from_note(note: ApubNoteModel) -> Self {
        let create_activity_id = {
            let note_url = note.id.inner().clone();
            Url::parse(format!("{note_url}/create").as_str()).unwrap()
        };
        Self {
            id: create_activity_id,
            kind: CreateType::Create,
            actor: note.attributed_to.clone(),
            to: note.to.clone(),
            cc: note.cc.clone(),
            object: CreatableObject::Note(note),
        }
    }
}

#[async_trait]
impl ActivityHandler for CreateActivity {
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
        match self.object {
            CreatableObject::Note(ref note) => {
                actor_check(self.actor(), note.attributed_to.inner())?;
                note.validate()?;
            }
        }

        Ok(())
    }

    async fn receive(self, data: &Data<Self::DataType>) -> Result<(), Self::Error> {
        match self.object {
            CreatableObject::Note(note) => {
                // save note
                let _ = NoteWithApubModel::from_json(note, data).await?;
            }
        }

        Ok(())
    }
}
