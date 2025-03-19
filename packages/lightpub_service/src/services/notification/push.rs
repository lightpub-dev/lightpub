use sea_orm::ActiveModelTrait;
use sea_orm::Set;
use sea_orm::SqlErr;
use serde::Deserialize;
use serde::Serialize;
use url::Url;

use crate::services::ServiceError;
use crate::{
    ServiceResult,
    services::{
        db::MaybeTxConn,
        id::{Identifier, UserID},
    },
};

#[derive(Debug, Clone, Deserialize)]
#[serde(rename_all = "camelCase")]
pub struct PushSubscription {
    endpoint: String,
    keys: PushSubscriptionKeys,
}

#[derive(Debug, Clone, Deserialize)]
#[serde(rename_all = "camelCase")]
pub struct PushSubscriptionKeys {
    p256dh: String,
    auth: String,
}

pub async fn register_push_subscription(
    conn: &MaybeTxConn,
    user_id: UserID,
    subscription: PushSubscription,
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

#[derive(Debug, Clone, Serialize)]
#[serde(rename_all = "camelCase")]
pub(crate) struct PushNotificationBody {
    pub title: String,
    pub body: String,
    pub url: Url,
}
