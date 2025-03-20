use activitypub_federation::{
    config::Data, fetch::object_id::ObjectId, kinds::activity::RejectType, traits::ActivityHandler,
};
use async_trait::async_trait;
use serde::{Deserialize, Serialize};
use url::Url;

use crate::{
    MyFederationData,
    services::{FederationServiceError, follow::unfollow_user, user::UserWithApubModel},
};

use super::{actor_check, follow::FollowActivity};

#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct RejectActivity {
    pub id: Url,
    #[serde(rename = "type")]
    pub kind: RejectType,
    pub actor: ObjectId<UserWithApubModel>,
    pub object: RejectableObject,
}

impl RejectActivity {
    pub fn new(actor: Url, object: impl Into<RejectableObject>) -> Self {
        let object = object.into();
        let undo_id = Url::parse(format!("{}/reject", object.id()).as_str()).unwrap();
        Self {
            id: undo_id,
            kind: RejectType::Reject,
            actor: ObjectId::from(actor),
            object,
        }
    }
}

#[derive(Debug, Clone, Serialize, Deserialize)]
#[serde(untagged)]
pub enum RejectableObject {
    Follow(FollowActivity),
}

impl From<FollowActivity> for RejectableObject {
    fn from(follow: FollowActivity) -> Self {
        RejectableObject::Follow(follow)
    }
}

#[async_trait]
impl ActivityHandler for RejectActivity {
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
            RejectableObject::Follow(follow) => actor_check(self.actor(), follow.object.inner())?, // actor == followed user
        }

        Ok(())
    }

    async fn receive(self, data: &Data<Self::DataType>) -> Result<(), Self::Error> {
        match self.object {
            RejectableObject::Follow(follow) => {
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
        }

        Ok(())
    }
}

impl RejectableObject {
    pub fn id(&self) -> &Url {
        match self {
            RejectableObject::Follow(follow) => follow.id(),
        }
    }

    pub fn actor(&self) -> &ObjectId<UserWithApubModel> {
        match self {
            RejectableObject::Follow(follow) => &follow.actor,
        }
    }
}
