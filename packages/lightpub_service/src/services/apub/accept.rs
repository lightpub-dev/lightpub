use activitypub_federation::{
    config::Data, fetch::object_id::ObjectId, kinds::activity::AcceptType, traits::ActivityHandler,
};
use async_trait::async_trait;
use serde::{Deserialize, Serialize};
use url::Url;

use crate::{
    MyFederationData,
    services::{FederationServiceError, follow::accept_pending_follow, user::UserWithApubModel},
};

use super::{actor_check, follow::FollowActivity};

#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct AcceptActivity {
    pub id: Url,
    #[serde(rename = "type")]
    pub kind: AcceptType,
    pub actor: ObjectId<UserWithApubModel>,
    pub object: AcceptableObject,
}

impl AcceptActivity {
    pub fn new(actor: Url, object: impl Into<AcceptableObject>) -> Self {
        let object = object.into();
        let accept_id = Url::parse(format!("{}/accept", object.id()).as_str()).unwrap();
        Self {
            id: accept_id,
            kind: AcceptType::Accept,
            actor: ObjectId::from(actor),
            object,
        }
    }
}

#[derive(Debug, Clone, Serialize, Deserialize)]
#[serde(untagged)]
pub enum AcceptableObject {
    Follow(FollowActivity),
}

impl From<FollowActivity> for AcceptableObject {
    fn from(value: FollowActivity) -> Self {
        Self::Follow(value)
    }
}

impl AcceptableObject {
    pub fn id(&self) -> &Url {
        match self {
            AcceptableObject::Follow(follow) => follow.id(),
        }
    }

    pub fn actor(&self) -> &ObjectId<UserWithApubModel> {
        match self {
            AcceptableObject::Follow(follow) => &follow.actor,
        }
    }
}

#[async_trait]
impl ActivityHandler for AcceptActivity {
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
            AcceptableObject::Follow(follow) => actor_check(self.actor(), follow.object.inner())?, // actor == followed user
        }

        Ok(())
    }

    async fn receive(self, data: &Data<Self::DataType>) -> Result<(), Self::Error> {
        match self.object {
            AcceptableObject::Follow(follow) => {
                let actor = follow.actor.dereference(data).await?;
                let followed = follow.object.dereference(data).await?;
                accept_pending_follow(
                    data.conn(),
                    &data.rconn(),
                    data.qconn(),
                    actor.basic.id,
                    followed.basic.id,
                    data.base_url(),
                )
                .await?;
            }
        }

        Ok(())
    }
}
