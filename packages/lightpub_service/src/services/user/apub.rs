/*
Lightpub: a simple ActivityPub server
Copyright (C) 2025 tinaxd

This program is free software: you can redistribute it and/or modify
it under the terms of the GNU Affero General Public License as published
by the Free Software Foundation, either version 3 of the License, or
(at your option) any later version.

This program is distributed in the hope that it will be useful,
but WITHOUT ANY WARRANTY; without even the implied warranty of
MERCHANTABILITY or FITNESS FOR A PARTICULAR PURPOSE.  See the
GNU Affero General Public License for more details.

You should have received a copy of the GNU Affero General Public License
along with this program.  If not, see <https://www.gnu.org/licenses/>.
*/

use activitypub_federation::config::Data;
use activitypub_federation::fetch::object_id::ObjectId;
use activitypub_federation::protocol::public_key::PublicKey;
use activitypub_federation::traits::{Actor, Object};
use actix_web::http::StatusCode;
use async_trait::async_trait;
use chrono::{DateTime, Utc};
use expected_error_derive::ExpectedError;
use nestify::nest;
use sea_orm::EntityTrait;
use sea_orm::{entity::*, query::*};
use serde::{Deserialize, Serialize};
use thiserror::Error;
use tracing::warn;
use url::Url;

use crate::MyFederationData;
use crate::utils::sanitize::CleanString;

use crate::services::ServiceError;

use super::super::auth::{validate_nickname, validate_username};
use super::super::db::MaybeTxConn;
use super::super::kv::KVObject;
use super::super::upload::register_remote_upload;
use super::super::{FederationServiceError, MapToUnknown};
use super::super::{
    ServiceResult,
    db::Conn,
    id::{Identifier, UserID},
};
use super::{
    SimpleUserModel, UserSpecifier, get_apubuser_by_id, get_apubuser_by_spec, get_user_id_from_url,
    invalidate_user_cache,
};

/// Apub JSON からユーザー情報を更新または新規追加する。
/// Apub JSON はこの関数を呼び出す前に検証されている必要がある。
pub async fn upsert_verified_apub_user(
    conn: &Conn,
    rconn: &KVObject,
    client: &reqwest_middleware::ClientWithMiddleware,
    user: &ApubUserModel,
    my_domain: &str,
    base_url: &Url,
) -> ServiceResult<UserWithApubModel> {
    let url = user.id.inner();

    let tx = conn.as_tx().await?.into();
    let user_id = get_user_id_from_url(&tx, url, my_domain).await?;

    let user_id = match user_id {
        None => insert_verified_apub_user(&tx, client, url, user).await?,
        Some(user_id) => update_verified_apub_user(&tx, rconn, client, user_id, user).await?,
    };

    tx.commit().await?;

    get_apubuser_by_id(&MaybeTxConn::Conn(conn.clone()), user_id, base_url)
        .await
        .map(|u| u.expect("user should exist"))
}

async fn set_user_active_model_from_apub(
    conn: &MaybeTxConn,
    client: &reqwest_middleware::ClientWithMiddleware,
    model: &mut entity::user::ActiveModel,
    pubkey_model: &mut entity::remote_public_key::ActiveModel,
    user: &ApubUserModel,
    user_id: UserID,
    user_url: &Url,
) {
    let fetched_at = Utc::now();
    let auto_follow_accept = match user.manually_approves_followers {
        Some(b) => !b,
        None => true,
    };

    let bio =
        CleanString::clean(user.summary.as_ref().map(|s| s.as_str()).unwrap_or("")).into_inner();

    let avatar_upload_id = if let Some(icon) = &user.icon {
        match register_remote_upload(conn, &icon.url, client).await {
            Ok(upload_id) => Some(upload_id),
            Err(e) => {
                warn!("failed to fetch avatar (skipped): {e}");
                None
            }
        }
    } else {
        None
    };

    model.id = Set(user_id.as_db());
    model.username = Set(user.preferred_username.clone());
    model.domain = Set(user_url.domain().unwrap().to_string());
    model.avatar = Set(avatar_upload_id.map(|u| u.as_db()));
    model.nickname = Set(user
        .name
        .clone()
        .unwrap_or_else(|| user.preferred_username.clone()));
    model.url = Set(Some(user_url.to_string()));
    model.view_url = Set(user.url.clone().map(|u| u.to_string()));
    model.bio = Set(bio);
    model.created_at = Set(user.published.map(|d| d.naive_utc()));
    model.auto_follow_accept = Set(auto_follow_accept as i8);
    model.is_bot = Set(user.kind.is_bot() as i8);
    model.shared_inbox = Set(user
        .endpoints
        .as_ref()
        .map(|e| e.shared_inbox.as_ref())
        .flatten()
        .map(|u| u.to_string()));
    model.inbox = Set(Some(user.inbox.to_string()));
    model.outbox = Set(Some(user.outbox.to_string()));
    model.fetched_at = Set(Some(fetched_at.naive_utc()));

    let pubkey = &user.public_key;
    pubkey_model.key_id = Set(pubkey.id.clone());
    pubkey_model.owner_id = Set(user_id.as_db());
    pubkey_model.public_key = Set(pubkey.public_key_pem.clone());
}

async fn insert_verified_apub_user(
    tx: &MaybeTxConn,
    client: &reqwest_middleware::ClientWithMiddleware,
    user_url: &Url,
    user: &ApubUserModel,
) -> ServiceResult<UserID> {
    let user_id = UserID::new_random();

    let mut user_model: entity::user::ActiveModel = Default::default();
    let mut pubkey_model: entity::remote_public_key::ActiveModel = Default::default();
    set_user_active_model_from_apub(
        tx,
        client,
        &mut user_model,
        &mut pubkey_model,
        user,
        user_id,
        user_url,
    )
    .await;
    user_model.insert(tx).await.map_err_unknown()?;
    pubkey_model.insert(tx).await.map_err_unknown()?;

    Ok(user_id)
}

async fn update_verified_apub_user(
    tx: &MaybeTxConn,
    rconn: &KVObject,
    client: &reqwest_middleware::ClientWithMiddleware,
    user_id: UserID,
    user: &ApubUserModel,
) -> ServiceResult<UserID> {
    invalidate_user_cache(rconn, user_id).await?;

    entity::remote_public_key::Entity::delete_many()
        .filter(entity::remote_public_key::Column::OwnerId.eq(user_id.as_db()))
        .exec(tx)
        .await
        .map_err_unknown()?;

    let user_model = entity::user::Entity::find_by_id(user_id.as_db())
        .one(tx)
        .await
        .map_err_unknown()?
        .expect("user should exist");

    let mut user_model = user_model.into_active_model();
    let mut pubkey_model: entity::remote_public_key::ActiveModel = Default::default();
    set_user_active_model_from_apub(
        tx,
        client,
        &mut user_model,
        &mut pubkey_model,
        user,
        user_id,
        user.id.inner(),
    )
    .await;
    user_model.update(tx).await.map_err_unknown()?;
    pubkey_model.insert(tx).await.map_err_unknown()?;

    Ok(user_id)
}

#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct UserWithApubModel {
    pub basic: SimpleUserModel,
    pub apub: UserApubData,
}

impl UserWithApubModel {
    pub fn is_remote(&self) -> bool {
        self.basic.is_remote()
    }

    pub fn is_local(&self) -> bool {
        self.basic.is_local()
    }

    pub fn shared_inbox_or_inbox(&self) -> &Url {
        self.apub
            .shared_inbox
            .as_ref()
            .unwrap_or_else(|| &self.apub.inbox)
    }
}

#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct UserApubData {
    pub url: Url,
    pub view_url: Option<Url>,
    pub inbox: Url,
    pub outbox: Url,
    pub shared_inbox: Option<Url>,
    pub followers: Option<Url>,
    pub following: Option<Url>,
    pub private_key: Option<String>,
    pub public_key: String,
    pub is_bot: bool,
    pub auto_follow_accept: bool,
    pub fetched_at: Option<DateTime<Utc>>,
    pub created_at: Option<DateTime<Utc>>,
}

nest! {
    #[derive(Debug, Clone, Deserialize, Serialize)]*
    #[serde(rename_all = "camelCase")]*
    pub struct ApubUserModel {
        pub(crate) id: ObjectId<UserWithApubModel>,
        #[serde(rename = "type")]
        pub(crate) kind: ApubUserKind,
        pub(crate) preferred_username: String,
        pub(crate) name: Option<String>,
        #[serde(skip_serializing_if = "Option::is_none")]
        pub(crate) summary: Option<String>,
        pub(crate) inbox: Url,
        pub(crate) outbox: Url,
        pub(crate) following: Option<Url>,
        pub(crate) followers: Option<Url>,
        pub(crate) public_key: PublicKey,
        pub(crate) manually_approves_followers: Option<bool>,
        pub(crate) published: Option<DateTime<Utc>>,
        pub(crate) endpoints: Option<pub struct ApubUserEndpointsModel {
            pub(crate) shared_inbox: Option<Url>,
        }>,
        pub(crate) icon: Option<pub struct ApubUserIconModel {
            #[serde(rename = "type")]
            pub(crate) kind: String,
            pub(crate) url: Url,
        }>,
        pub(crate) url: Option<Url>,
    }
}

#[derive(Debug, Clone, Copy, PartialEq, Eq, Serialize, Deserialize)]
pub enum ApubUserKind {
    #[serde(rename = "Person")]
    Person,
    #[serde(rename = "Application")]
    Application,
    #[serde(rename = "Service")]
    Service,
}

impl ApubUserKind {
    pub fn is_bot(&self) -> bool {
        match self {
            Self::Person => false,
            Self::Application => true,
            Self::Service => true,
        }
    }

    pub fn create(is_bot: bool) -> Self {
        if is_bot {
            Self::Application
        } else {
            Self::Person
        }
    }
}

#[derive(Debug, Clone, Error, ExpectedError)]
pub enum ApubUserValidationError {
    #[error("Bad domain")]
    #[ee(status(StatusCode::BAD_REQUEST))]
    BadDomain,
    #[error("Bad public key")]
    #[ee(status(StatusCode::BAD_REQUEST))]
    BadPublicKey,
}

#[async_trait]
impl Object for UserWithApubModel {
    type Error = FederationServiceError;
    type DataType = MyFederationData;
    type Kind = ApubUserModel;

    async fn read_from_id(
        object_id: Url,
        data: &Data<Self::DataType>,
    ) -> Result<Option<Self>, Self::Error> {
        let user_spec = UserSpecifier::url(object_id);
        let apub = get_apubuser_by_spec(
            &data.maybe_conn(),
            &user_spec,
            &data.my_domain(),
            data.base_url(),
        )
        .await?;

        Ok(apub)
    }

    async fn into_json(self, data: &Data<Self::DataType>) -> Result<Self::Kind, Self::Error> {
        let user_url = self.apub.url;

        let avatar_obj = self.basic.avatar.as_ref().map(|a| {
            let avatar_url = data
                .base_url()
                .join(format!("upload/{}", a).as_str())
                .unwrap();
            ApubUserIconModel {
                kind: "Image".to_string(),
                url: avatar_url,
            }
        });

        Ok(ApubUserModel {
            id: ObjectId::parse(&user_url.to_string()).unwrap(),
            kind: ApubUserKind::create(self.apub.is_bot),
            preferred_username: self.basic.username,
            name: Some(self.basic.nickname),
            summary: Some(self.basic.bio),
            inbox: self.apub.inbox,
            outbox: self.apub.outbox,
            following: self.apub.following,
            followers: self.apub.followers,
            public_key: PublicKey {
                id: format!("{}#main-key", user_url),
                owner: user_url,
                public_key_pem: self.apub.public_key,
            },
            manually_approves_followers: Some(!self.apub.auto_follow_accept),
            published: self.apub.created_at,
            endpoints: Some(ApubUserEndpointsModel {
                shared_inbox: self.apub.shared_inbox,
            }),
            url: self.apub.view_url,
            icon: avatar_obj,
        })
    }

    async fn verify(
        json: &Self::Kind,
        expected_domain: &Url,
        _data: &Data<Self::DataType>,
    ) -> Result<(), Self::Error> {
        let json_id_domain = json
            .id
            .inner()
            .domain()
            .ok_or(ServiceError::known(ApubUserValidationError::BadDomain))?;
        if json_id_domain != expected_domain.domain().expect("expected domain") {
            return Err(ServiceError::known(ApubUserValidationError::BadDomain).into());
        }

        json.validate()?;

        Ok(())
    }

    async fn from_json(json: Self::Kind, data: &Data<Self::DataType>) -> Result<Self, Self::Error> {
        upsert_verified_apub_user(
            data.conn(),
            &data.rconn(),
            data.proxy_client(),
            &json,
            &data.my_domain(),
            data.base_url(),
        )
        .await
        .map_err(|e| e.into())
    }

    async fn delete(self, _data: &Data<Self::DataType>) -> Result<(), Self::Error> {
        // TODO: delete remote user on request
        Ok(())
    }

    fn last_refreshed_at(&self) -> Option<DateTime<Utc>> {
        self.apub.fetched_at.clone()
    }
}

impl Actor for UserWithApubModel {
    fn id(&self) -> Url {
        self.apub.url.clone()
    }
    fn public_key_pem(&self) -> &str {
        self.apub.public_key.as_str()
    }
    fn private_key_pem(&self) -> Option<String> {
        self.apub.private_key.clone()
    }
    fn inbox(&self) -> Url {
        self.apub.inbox.clone()
    }
}

impl ApubUserModel {
    pub fn validate(&self) -> ServiceResult<()> {
        let domain_eq = |u1: &Url, u2: &Url| -> ServiceResult<bool> {
            let u1_domain = u1
                .domain()
                .ok_or(ServiceError::known(ApubUserValidationError::BadDomain))?;
            let u2_domain = u2
                .domain()
                .ok_or(ServiceError::known(ApubUserValidationError::BadDomain))?;
            Ok(u1_domain == u2_domain)
        };

        if !domain_eq(self.id.inner(), &self.inbox)? {
            return Err(ServiceError::known(ApubUserValidationError::BadDomain));
        }
        if !domain_eq(self.id.inner(), &self.outbox)? {
            return Err(ServiceError::known(ApubUserValidationError::BadDomain));
        }
        if let Some(following) = &self.following {
            if !domain_eq(self.id.inner(), following)? {
                return Err(ServiceError::known(ApubUserValidationError::BadDomain));
            }
        }
        if let Some(followers) = &self.followers {
            if !domain_eq(self.id.inner(), followers)? {
                return Err(ServiceError::known(ApubUserValidationError::BadDomain));
            }
        }
        if self.public_key.owner != *self.id.inner() {
            return Err(ServiceError::known(ApubUserValidationError::BadPublicKey));
        }
        let public_key_id = Url::parse(&self.public_key.id)
            .map_err(|_| ServiceError::known(ApubUserValidationError::BadPublicKey))?;
        if !domain_eq(&public_key_id, self.id.inner())? {
            return Err(ServiceError::known(ApubUserValidationError::BadPublicKey));
        }

        validate_username(&self.preferred_username)?;
        let nickname = self.name.as_ref().unwrap_or(&self.preferred_username);
        validate_nickname(nickname)?;

        Ok(())
    }
}

#[test]
fn test_parse_misskey_user() {
    use serde_json::json;
    let json = json!({"@context":["https://www.w3.org/ns/activitystreams","https://w3id.org/security/v1",{"Key":"sec:Key","manuallyApprovesFollowers":"as:manuallyApprovesFollowers","sensitive":"as:sensitive","Hashtag":"as:Hashtag","quoteUrl":"as:quoteUrl","toot":"http://joinmastodon.org/ns#","Emoji":"toot:Emoji","featured":"toot:featured","discoverable":"toot:discoverable","schema":"http://schema.org#","PropertyValue":"schema:PropertyValue","value":"schema:value","misskey":"https://misskey-hub.net/ns#","_misskey_content":"misskey:_misskey_content","_misskey_quote":"misskey:_misskey_quote","_misskey_reaction":"misskey:_misskey_reaction","_misskey_votes":"misskey:_misskey_votes","_misskey_summary":"misskey:_misskey_summary","isCat":"misskey:isCat","vcard":"http://www.w3.org/2006/vcard/ns#"}],"type":"Person","id":"https://misskey.tinax.local/users/9r70xhde0mav0001","inbox":"https://misskey.tinax.local/users/9r70xhde0mav0001/inbox","outbox":"https://misskey.tinax.local/users/9r70xhde0mav0001/outbox","followers":"https://misskey.tinax.local/users/9r70xhde0mav0001/followers","following":"https://misskey.tinax.local/users/9r70xhde0mav0001/following","featured":"https://misskey.tinax.local/users/9r70xhde0mav0001/collections/featured","sharedInbox":"https://misskey.tinax.local/inbox","endpoints":{"sharedInbox":"https://misskey.tinax.local/inbox"},"url":"https://misskey.tinax.local/@missuser","preferredUsername":"missuser","name":"missuser dayo","summary":"<p>misskey bio</p>","_misskey_summary":"misskey bio","icon":null,"image":null,"tag":[],"manuallyApprovesFollowers":false,"discoverable":true,"publicKey":{"id":"https://misskey.tinax.local/users/9r70xhde0mav0001#main-key","type":"Key","owner":"https://misskey.tinax.local/users/9r70xhde0mav0001","publicKeyPem":"-----BEGIN PUBLIC KEY-----\nMIIBIjANBgkqhkiG9w0BAQEFAAOCAQ8AMIIBCgKCAQEA9RbL++NOcIcKPpB+0N9t\nkG7LQxDQzJOzlC9r8aEiWKGdh+DfnmYGuTP1yv8pV9eSYffKQLO73enzzrbGXVT7\ns9efaqmkdF0oaxQKm8wAW8HPqw518R/tuIflNOJ5l59Juju34MyvOqY2QdbOFZLG\n6E4xjAvmTQVn8TOXcehcIxVzc8jh7MmoAUWm2m/LYeBOlyo67bElD1JH4Iw1kuWg\n437CNKfaIDhW+W9H0veQ1l+Y4jnWqpvjlmJ33d6/MR/fPk7VQZlJR2K/8p9iFa21\nkWgjx9Hv481+poksQdUdbT0ZciDtbSx7p+PW2DuAU2Zsv+zrurcfLeDd16ml2vUB\nSQIDAQAB\n-----END PUBLIC KEY-----\n"},"isCat":false});
    let user: ApubUserModel = serde_json::from_value(json).unwrap();
    assert_eq!(
        user.id.inner().as_str(),
        "https://misskey.tinax.local/users/9r70xhde0mav0001"
    );
}
