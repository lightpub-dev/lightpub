use std::borrow::Cow;
use std::sync::OnceLock;

use base64::Engine;
use base64::prelude::BASE64_STANDARD;
use derive_more::From;
use derive_new::new;
use expected_error::ExpectedError;
use sea_orm::ActiveModelTrait;
use sea_orm::Set;
use sea_orm::SqlErr;
use sea_orm::prelude::*;
use serde::Serialize;
use thiserror::Error;
use url::Url;
use web_push::IsahcWebPushClient;
use web_push::PartialVapidSignatureBuilder;
pub use web_push::SubscriptionInfo;
use web_push::SubscriptionKeys;
use web_push::WebPushClient;
use web_push::WebPushError;
use web_push::WebPushMessageBuilder;

use crate::services::INTERNAL_SERVER_ERROR_TEXT;
use crate::services::MapToUnknown;
use crate::services::ServiceError;
use crate::{
    ServiceResult,
    services::{
        db::MaybeTxConn,
        id::{Identifier, UserID},
    },
};

use super::NotificationBodyData;

pub async fn register_push_subscription(
    conn: &MaybeTxConn,
    user_id: UserID,
    subscription: SubscriptionInfo,
) -> ServiceResult<()> {
    let model = entity::push_notification::ActiveModel {
        user_id: Set(user_id.as_db()),
        endpoint: Set(subscription.endpoint),
        p256dh: Set(subscription.keys.p256dh),
        auth: Set(subscription.keys.auth),
        ..Default::default()
    };
    let result = model.save(conn).await;
    match result {
        Ok(_) => {}
        Err(e) if matches!(e.sql_err(), Some(SqlErr::UniqueConstraintViolation(_))) => {} // ignore duplicate
        Err(e) => return Err(ServiceError::unknown(e)),
    }

    Ok(())
}

#[derive(Debug, Clone)]
pub(crate) struct UserWPSubscription {
    pub subscription_id: i32,
    pub subscription: SubscriptionInfo,
}

pub(crate) async fn get_subscriptions_for_user(
    conn: &MaybeTxConn,
    user_id: UserID,
) -> ServiceResult<Vec<UserWPSubscription>> {
    let rows = entity::push_notification::Entity::find()
        .filter(entity::push_notification::Column::UserId.eq(user_id.as_db()))
        .all(conn)
        .await
        .map_err_unknown()?;

    let subs = rows
        .into_iter()
        .map(|s| UserWPSubscription {
            subscription_id: s.id,
            subscription: SubscriptionInfo {
                endpoint: s.endpoint,
                keys: SubscriptionKeys {
                    auth: s.auth,
                    p256dh: s.p256dh,
                },
            },
        })
        .collect();

    Ok(subs)
}

pub(crate) async fn delete_subscription_id(
    conn: &MaybeTxConn,
    subscription_id: i32,
) -> ServiceResult<()> {
    entity::push_notification::Entity::delete_by_id(subscription_id)
        .exec(conn)
        .await
        .map_err_unknown()?;
    Ok(())
}

#[derive(Debug, Clone, Serialize)]
#[serde(rename_all = "camelCase")]
pub(crate) struct PushNotificationBody {
    pub title: String,
    pub body: String,
    pub url: Url,
}

impl PushNotificationBody {
    pub(crate) fn new_from_notification_body(
        base_url: &Url,
        body: NotificationBodyData,
    ) -> ServiceResult<PushNotificationBody> {
        let (title, body, url) = match body {
            NotificationBodyData::Followed(u) => {
                let title = format!("フォローされました");
                let body = format!("{} があなたをフォローしました", u.basic.nickname);
                let url = base_url
                    .join(&format!("/client/user/{}", u.basic.id))
                    .unwrap();
                (title, body, url)
            }
            NotificationBodyData::FollowRequested(u) => {
                let title = format!("フォローリクエストが届いています");
                let body = format!(
                    "{} があなたをフォローしたいとリクエストしています",
                    u.basic.nickname
                );
                let url = base_url
                    .join(&format!("/client/user/{}", u.basic.id))
                    .unwrap();
                (title, body, url)
            }
            NotificationBodyData::FollowAccepted(u) => {
                let title = format!("フォローリクエストが承認されました");
                let body = format!(
                    "{} があなたのフォローリクエストを承認しました",
                    u.basic.nickname
                );
                let url = base_url
                    .join(&format!("/client/user/{}", u.basic.id))
                    .unwrap();
                (title, body, url)
            }
            NotificationBodyData::Mentioned { user, note } => {
                let title = format!("メンションされました");
                let body = format!("{} があなたをメンションしました", user.basic.nickname);
                let url = note.view_url.clone();
                (title, body, url)
            }
            NotificationBodyData::Renoted { user, renoted_note } => {
                let title = format!("リノートされました");
                let body = format!("{} があなたのノートをリノートしました", user.basic.nickname);
                let url = renoted_note.view_url.clone();
                (title, body, url)
            }
            NotificationBodyData::Replied {
                author, reply_note, ..
            } => {
                let title = format!("リプライされました");
                let body = format!(
                    "{} があなたのノートにリプライしました",
                    author.basic.nickname
                );
                let url = reply_note.view_url.clone();
                (title, body, url)
            }
        };

        Ok(PushNotificationBody { title, body, url })
    }
}

#[derive(Clone, new)]
pub struct WPClient {
    client: IsahcWebPushClient,
    vapid_builder: PartialVapidSignatureBuilder,

    #[new(default)]
    _public_key_cache: OnceLock<String>,
}

impl std::fmt::Debug for WPClient {
    fn fmt(&self, f: &mut std::fmt::Formatter<'_>) -> std::fmt::Result {
        f.debug_struct("WPClient")
            .field("client", &"<IsahcWebPushClient>")
            .field("vapid_builder", &"<PartialVapidSignatureBuilder>")
            .finish()
    }
}

#[derive(Debug)]
pub(crate) enum PushSendResult {
    Success,
    Failed(PushNotificationSendError),
}

#[derive(Debug, Error, From)]
#[error("Failed to send push notification: {0}")]
pub(crate) struct PushNotificationSendError(WebPushError);

impl PushNotificationSendError {
    // pub fn into_inner(self) -> WebPushError {
    //     self.0
    // }

    // pub fn inner(&self) -> &WebPushError {
    //     &self.0
    // }

    pub fn should_disable_endpoint(&self) -> bool {
        match &self.0 {
            WebPushError::EndpointNotFound(_) => true,
            WebPushError::EndpointNotValid(_) => true,
            _ => false,
        }
    }
}

impl ExpectedError for PushNotificationSendError {
    fn msg(&self) -> std::borrow::Cow<str> {
        Cow::Borrowed(INTERNAL_SERVER_ERROR_TEXT)
    }

    fn status(&self) -> expected_error::StatusCode {
        expected_error::StatusCode::INTERNAL_SERVER_ERROR
    }
}

impl WPClient {
    fn generate_public_key(&self) -> String {
        let bytes = self.vapid_builder.get_public_key();
        BASE64_STANDARD.encode(&bytes)
    }

    pub fn public_key(&self) -> &str {
        self._public_key_cache
            .get_or_init(|| self.generate_public_key())
    }

    pub(crate) async fn try_send(
        &self,
        subscription: &SubscriptionInfo,
        body: &PushNotificationBody,
    ) -> ServiceResult<PushSendResult> {
        let sig = self
            .vapid_builder
            .clone()
            .add_sub_info(subscription)
            .build()
            .map_err_unknown()?;

        let mut builder = WebPushMessageBuilder::new(subscription);
        let content = serde_json::to_vec(body).map_err_unknown()?;
        builder.set_payload(web_push::ContentEncoding::Aes128Gcm, &content);
        builder.set_vapid_signature(sig);
        let message = builder.build().map_err_unknown()?;

        let result = self.client.send(message).await;
        match result {
            Ok(_) => Ok(PushSendResult::Success),
            Err(e) => Ok(PushSendResult::Failed(PushNotificationSendError(e))),
        }
    }
}
