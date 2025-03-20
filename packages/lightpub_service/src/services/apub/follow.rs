use activitypub_federation::{
    config::Data, fetch::object_id::ObjectId, kinds::activity::FollowType, traits::ActivityHandler,
};
use async_trait::async_trait;
use serde::{Deserialize, Serialize};
use url::Url;

use crate::{
    MyFederationData,
    services::{FederationServiceError, follow::follow_user, user::UserWithApubModel},
};

#[derive(Debug, Clone, Serialize, Deserialize)]
#[serde(rename_all = "camelCase")]
pub struct FollowActivity {
    pub actor: ObjectId<UserWithApubModel>,
    pub object: ObjectId<UserWithApubModel>,
    #[serde(rename = "type")]
    pub kind: FollowType,
    pub id: Url,
}

impl FollowActivity {
    pub fn new(id: i32, follower: Url, followed: Url, base_url: &Url) -> Self {
        let id = base_url.join(format!("follow/{}", id).as_str()).unwrap();
        FollowActivity {
            id,
            kind: FollowType::Follow,
            actor: ObjectId::from(follower),
            object: ObjectId::from(followed),
        }
    }

    pub fn new_from_url(url: Url, follower: Url, followed: Url) -> Self {
        FollowActivity {
            id: url,
            kind: FollowType::Follow,
            actor: ObjectId::from(follower),
            object: ObjectId::from(followed),
        }
    }
}

#[async_trait]
impl ActivityHandler for FollowActivity {
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
        let actor = self.actor.dereference(data).await?;
        let followed = self.object.dereference(data).await?;
        follow_user(
            data.conn(),
            &data.rconn(),
            data.qconn(),
            data.wp(),
            actor.basic.id,
            followed.basic.id,
            data.base_url(),
        )
        .await?;
        Ok(())
    }
}
