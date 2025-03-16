use activitypub_federation::{
    config::Data,
    fetch::object_id::ObjectId,
    kinds::activity::UpdateType,
    traits::{ActivityHandler, Object},
};
use async_trait::async_trait;
use chrono::Utc;
use serde::{Deserialize, Serialize};
use url::Url;

use crate::{
    MyFederationData,
    services::{
        FederationServiceError,
        note::{ApubNoteModel, NoteWithApubModel},
        user::{ApubUserModel, UserWithApubModel},
    },
};

use super::{PUBLIC_URL, actor_check};

#[derive(Debug, Clone, Serialize, Deserialize)]
#[serde(rename_all = "camelCase")]
pub struct UpdateActivity {
    pub id: Url,
    #[serde(rename = "type")]
    pub kind: UpdateType,
    pub actor: ObjectId<UserWithApubModel>,
    #[serde(default)]
    pub to: Vec<Url>,
    pub object: UpdatableObject,
}

#[derive(Debug, Clone, Serialize, Deserialize)]
#[serde(untagged)]
pub enum UpdatableObject {
    Note(ApubNoteModel),
    User(ApubUserModel),
}

impl UpdateActivity {
    pub fn from_note(note: ApubNoteModel) -> Self {
        let update_activity_id = {
            let note_url = note.id.inner().clone();
            let time = Utc::now().timestamp_millis();
            Url::parse(format!("{note_url}/update/{time}").as_str()).unwrap()
        };
        Self {
            id: update_activity_id,
            kind: UpdateType::Update,
            actor: note.attributed_to.clone(),
            to: vec![(&*PUBLIC_URL).clone()],
            object: UpdatableObject::Note(note),
        }
    }

    pub fn from_user(user: ApubUserModel) -> Self {
        let update_activity_id = {
            let user_url = user.id.inner().clone();
            let time = Utc::now().timestamp_millis();
            Url::parse(format!("{user_url}/update/{time}").as_str()).unwrap()
        };
        Self {
            id: update_activity_id,
            kind: UpdateType::Update,
            actor: user.id.clone(),
            to: vec![(&*PUBLIC_URL).clone()],
            object: UpdatableObject::User(user),
        }
    }
}

#[async_trait]
impl ActivityHandler for UpdateActivity {
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
            UpdatableObject::Note(ref note) => {
                actor_check(self.actor(), note.attributed_to.inner())?;
                note.validate()?;
            }
            UpdatableObject::User(ref user) => {
                actor_check(self.actor(), user.id.inner())?;
                user.validate()?;
            }
        }

        Ok(())
    }

    async fn receive(self, data: &Data<Self::DataType>) -> Result<(), Self::Error> {
        match self.object {
            UpdatableObject::Note(note) => {
                // upsert note
                let _ = NoteWithApubModel::from_json(note, data).await?;
            }
            UpdatableObject::User(user) => {
                // upsert user
                let _ = UserWithApubModel::from_json(user, data).await?;
            }
        }

        Ok(())
    }
}
