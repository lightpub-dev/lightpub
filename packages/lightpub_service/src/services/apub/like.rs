use activitypub_federation::{
    config::Data, fetch::object_id::ObjectId, kinds::activity::LikeType, traits::ActivityHandler,
};
use async_trait::async_trait;
use serde::{Deserialize, Serialize};
use url::Url;

use crate::{
    MyFederationData,
    services::{
        FederationServiceError,
        note::{NoteWithApubModel, note_like_add},
        user::UserWithApubModel,
    },
};

#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct LikeActivity {
    pub id: Url,
    #[serde(rename = "type")]
    pub kind: LikeType,
    pub actor: ObjectId<UserWithApubModel>,
    pub object: LikeableObject,
}

impl LikeActivity {
    pub fn new(id: i32, actor: Url, object: impl Into<LikeableObject>, base_url: &Url) -> Self {
        let object = object.into();
        let like_id = base_url.join(&format!("like/{}", id)).unwrap();
        Self {
            id: like_id,
            kind: LikeType::Like,
            actor: ObjectId::from(actor),
            object,
        }
    }
}

#[derive(Debug, Clone, Serialize, Deserialize)]
#[serde(untagged)]
pub enum LikeableObject {
    Note(ObjectId<NoteWithApubModel>),
}

impl From<NoteWithApubModel> for LikeableObject {
    fn from(value: NoteWithApubModel) -> Self {
        Self::Note(ObjectId::from(value.apub.url))
    }
}

impl LikeableObject {
    pub fn note(id: impl Into<ObjectId<NoteWithApubModel>>) -> Self {
        Self::Note(id.into())
    }

    pub fn id(&self) -> &Url {
        match self {
            LikeableObject::Note(note) => note.inner(),
        }
    }
}

#[async_trait]
impl ActivityHandler for LikeActivity {
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
            LikeableObject::Note(note) => {
                let note = note.dereference(data).await?;
                let actor = self.actor.dereference(data).await?;
                note_like_add(
                    data.conn(),
                    data.qconn(),
                    actor.basic.id,
                    note.basic.id,
                    false,
                    &data.my_domain(),
                    data.base_url(),
                )
                .await?;
            }
        }

        Ok(())
    }
}
