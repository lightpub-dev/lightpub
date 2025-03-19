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

/// Route handlers for /notification
use actix_web::{get, post, web, HttpResponse, Responder};
use chrono::{DateTime, Utc};
use expected_error::StatusCode;
use lightpub_service::services::create_error_simple;
use lightpub_service::services::notification::push::{
    register_push_subscription, update_user_active_time,
    FRONTEND_UNREAD_NOTIFICATION_POLLING_INTERVAL,
};
use lightpub_service::services::notification::{
    get_related_notification_data, NotificationBodyData,
};
use nestify::nest;
use serde::Serialize;
use url::Url;
use web_push::SubscriptionInfo;

use super::auth::AuthedUser;
use crate::api::auth::middleware_auth_jwt_required;
use crate::{
    template::{
        render_template, NotifyFollowRequestedBody, NotifyFollowRequestedBodyData,
        NotifyFollowedBody, NotifyFollowedBodyData, NotifyMentionedBody, NotifyMentionedBodyData,
        NotifyNotifyBase, NotifyRenotedBody, NotifyRenotedBodyData, NotifyRepliedBody,
        NotifyRepliedBodyData, PartsNotifyList, PartsNotifyListEntry, Template,
    },
    AppState,
};
use actix_web::middleware::from_fn;
use lightpub_service::services::{
    db::MaybeTxConn,
    id::NotificationID,
    kv::KVObject,
    notification::{
        count_unread_notifications, get_notifications, mark_notification_read,
        mark_notification_read_all, Notification, NotificationBody,
    },
    ServiceResult,
};
use lightpub_service::try_opt_res;
#[post("/all/read", wrap = "from_fn(middleware_auth_jwt_required)")]
pub async fn api_read_all_notifications(
    st: web::Data<AppState>,
    auth: web::ReqData<AuthedUser>,
) -> ServiceResult<impl Responder> {
    let user_id = auth.user_id_unwrap();

    mark_notification_read_all(st.conn(), user_id).await?;

    Ok(HttpResponse::NoContent()
        .insert_header(("hx-refresh", "true"))
        .finish())
}

#[post(
    "/{notification_id}/read",
    wrap = "from_fn(middleware_auth_jwt_required)"
)]
pub async fn api_read_notification(
    st: web::Data<AppState>,
    notification_id: web::Path<NotificationID>,
    auth: web::ReqData<AuthedUser>,
) -> ServiceResult<impl Responder> {
    let user_id = auth.user_id_unwrap();

    mark_notification_read(st.conn(), notification_id.into_inner(), user_id).await?;

    Ok(HttpResponse::NoContent()
        .insert_header(("hx-refresh", "true"))
        .finish())
}

#[get("/all", wrap = "from_fn(middleware_auth_jwt_required)")]
pub async fn api_get_notifications(
    st: web::Data<AppState>,
    auth: web::ReqData<AuthedUser>,
) -> ServiceResult<impl Responder> {
    let user_id = auth.user_id_unwrap();

    let notifications = get_notifications(st.conn(), user_id).await?;

    let mut ns = vec![];
    for n in notifications {
        let nt =
            create_notification_template(&st.maybe_conn(), &st.rconn(), st.base_url(), n).await?;
        if let Some(nt) = nt {
            ns.push(nt);
        }
    }

    let temp = Template::PartsNotifyList(PartsNotifyList { data: ns });

    render_template(st.template(), &temp)
}

async fn create_notification_template(
    conn: &MaybeTxConn,
    rconn: &KVObject,
    base_url: &Url,
    n: Notification,
) -> ServiceResult<Option<PartsNotifyListEntry>> {
    let base = |template_name: String| NotifyNotifyBase {
        id: n.id,
        template_name: template_name.to_owned(),
        icon_url: None, // TODO: notification icon
        created_at: n.created_at,
        read_at: n.read_at,
    };
    let data = try_opt_res!(get_notification_data(conn, rconn, base, base_url, &n.body).await?);

    Ok(Some(data))
}

async fn get_notification_data(
    conn: &MaybeTxConn,
    rconn: &KVObject,
    base_func: impl FnOnce(String) -> NotifyNotifyBase,
    base_url: &Url,
    raw_body: &NotificationBody,
) -> ServiceResult<Option<PartsNotifyListEntry>> {
    let body = get_related_notification_data(conn, rconn, base_url, raw_body).await?;

    let body = match body {
        None => return Ok(None),
        Some(b) => b,
    };

    match body {
        NotificationBodyData::Followed(follower) => {
            Ok(Some(PartsNotifyListEntry::Followed(NotifyFollowedBody {
                base: base_func("parts/notify/followed".to_owned()),
                data: NotifyFollowedBodyData {
                    follower_nickname: follower.basic.nickname,
                    follower_url: format!("/client/user/{}", follower.basic.specifier),
                },
            })))
        }
        NotificationBodyData::FollowRequested(follower) => Ok(Some(
            PartsNotifyListEntry::FollowRequested(NotifyFollowRequestedBody {
                base: base_func("parts/notify/follow_requested".to_owned()),
                data: NotifyFollowRequestedBodyData {
                    follower_url: format!("/client/user/{}", follower.basic.specifier),
                    follower_nickname: follower.basic.nickname,
                },
            }),
        )),
        NotificationBodyData::Mentioned { user, note } => {
            Ok(Some(PartsNotifyListEntry::Mentioned(NotifyMentionedBody {
                base: base_func("parts/notify/mentioned".to_owned()),
                data: NotifyMentionedBodyData {
                    author_nickname: user.basic.nickname,
                    author_url: format!("/client/user/{}", user.basic.specifier),
                    note_url: note.view_url.to_string(),
                },
            })))
        }
        NotificationBodyData::Replied {
            author,
            replied_note,
            reply_note,
        } => Ok(Some(PartsNotifyListEntry::Replied(NotifyRepliedBody {
            base: base_func("parts/notify/replied".to_owned()),
            data: NotifyRepliedBodyData {
                author_url: format!("/client/user/{}", author.basic.id),
                author_nickname: author.basic.nickname,
                reply_note_url: reply_note.view_url.to_string(),
                replied_note_url: replied_note.view_url.to_string(),
            },
        }))),
        NotificationBodyData::Renoted { user, renoted_note } => {
            Ok(Some(PartsNotifyListEntry::Renoted(NotifyRenotedBody {
                base: base_func("parts/notify/renoted".to_owned()),
                data: NotifyRenotedBodyData {
                    author_url: format!("/client/user/{}", user.basic.id),
                    author_nickname: user.basic.nickname,
                    renoted_note_url: renoted_note.view_url.to_string(),
                },
            })))
        }
        _ => todo!(),
    }
}

#[get("/unread-count", wrap = "from_fn(middleware_auth_jwt_required)")]
pub async fn api_unread_notification_count(
    st: web::Data<AppState>,
    auth: web::ReqData<AuthedUser>,
) -> ServiceResult<impl Responder> {
    let user_id = auth.user_id_unwrap();
    let count = count_unread_notifications(st.conn(), user_id).await?;
    update_user_active_time(
        &st.rconn(),
        user_id,
        FRONTEND_UNREAD_NOTIFICATION_POLLING_INTERVAL,
    )
    .await?;

    let count_str = if count == 0 {
        "".to_string()
    } else {
        count.to_string()
    };
    Ok(HttpResponse::Ok().body(count_str))
}

nest! {
    #[derive(Debug, Clone, Serialize)]*
    #[serde(rename_all = "camelCase")]*
    struct NotificationListTemplate {
        data: Vec<struct NotificationTemplate {
            id: NotificationID,
            template_name: String,
            icon_url: Option<String>,
            data:
            #[serde(untagged)]
            enum NotificationData {
                Followed(struct NotificationFollowedData {
                    follower_url: String,
                    follower_nickname: String,
                }),
                FollowRequested(NotificationFollowedData),
                Mentioned(struct NotificationMentionedData {
                    author_url: String,
                    author_nickname: String,
                    note_url: String,
                }),
                Replied(struct NotificationRepliedData {
                    author_url: String,
                    author_nickname: String,
                    reply_note_url: String,
                    replied_note_url: String,
                }),
                Renoted(struct NotificationRenotedData {
                    author_url: String,
                    author_nickname: String,
                    renoted_note_url: String,
                })
            },
            created_at: DateTime<Utc>,
            read_at: Option<DateTime<Utc>>,
        }>
    }

}

#[post("/push/subscribe", wrap = "from_fn(middleware_auth_jwt_required)")]
pub async fn api_wp_subscribe(
    st: web::Data<AppState>,
    subscription: web::Json<SubscriptionInfo>,
    auth: web::ReqData<AuthedUser>,
) -> ServiceResult<impl Responder> {
    if st.wp().is_none() {
        return create_error_simple(
            StatusCode::BAD_REQUEST,
            "WebPush is disabled on this server",
        );
    }

    // Store subscription with user ID
    let user_id = auth.user_id_unwrap();
    register_push_subscription(&st.maybe_conn(), user_id, subscription.into_inner()).await?;
    Ok(HttpResponse::NoContent().finish())
}

#[get("/push/public-key")]
pub async fn api_wp_public_key(st: web::Data<AppState>) -> impl Responder {
    if let Some(wp) = st.wp() {
        Ok(HttpResponse::Ok().body(wp.public_key().to_owned()))
    } else {
        return create_error_simple(
            StatusCode::BAD_REQUEST,
            "WebPush is disabled on this server",
        );
    }
}
