use activitypub_federation::{
    config::Data,
    fetch::object_id::ObjectId,
    kinds::{activity::DeleteType, object::TombstoneType},
    traits::{ActivityHandler, Object},
};
use async_trait::async_trait;
use chrono::{DateTime, Utc};
use serde::{Deserialize, Serialize};
use url::Url;

use crate::{
    MyFederationData,
    services::{FederationServiceError, note::NoteWithApubModel, user::UserWithApubModel},
};

#[derive(Debug, Clone, Serialize, Deserialize)]
#[serde(rename_all = "camelCase")]
pub struct DeleteActivity {
    pub id: Url,
    #[serde(rename = "type")]
    pub kind: DeleteType,
    pub actor: ObjectId<UserWithApubModel>,
    pub published: Option<DateTime<Utc>>,
    pub object: Tombstone,
}

#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct Tombstone {
    pub id: Url,
    #[serde(rename = "type")]
    pub kind: TombstoneType,
}

impl DeleteActivity {
    pub fn from_note(
        target_note: ObjectId<NoteWithApubModel>,
        actor: ObjectId<UserWithApubModel>,
        deleted_at: DateTime<Utc>,
    ) -> Self {
        let delete_id = Url::parse(&format!("{}/delete", &target_note.inner())).unwrap();
        Self {
            id: delete_id,
            kind: DeleteType::Delete,
            actor,
            object: Tombstone {
                id: target_note.into_inner(),
                kind: TombstoneType::Tombstone,
            },
            published: Some(deleted_at),
        }
    }
}

#[async_trait]
impl ActivityHandler for DeleteActivity {
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
        // try to get object as note
        let note_id = ObjectId::<NoteWithApubModel>::from(self.object.id.clone());
        let note = note_id.dereference_local(data).await;

        use activitypub_federation::error::Error;
        match note {
            Ok(n) => n.delete(data).await,
            Err(FederationServiceError::FederationError(Error::NotFound)) => Ok(()),
            Err(e) => Err(e.into()),
        }
    }
}
