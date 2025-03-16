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
use nestify::nest;
use serde::Serialize;

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
    user::get_user_profile_by_id,
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
        let nt = create_notification_template(&st.maybe_conn(), &st.rconn(), n).await?;
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
    n: Notification,
) -> ServiceResult<Option<PartsNotifyListEntry>> {
    let base = |template_name: String| NotifyNotifyBase {
        id: n.id,
        template_name: template_name.to_owned(),
        icon_url: None, // TODO: notification icon
        created_at: n.created_at,
        read_at: n.read_at,
    };
    let data = try_opt_res!(get_notification_data(conn, rconn, base, &n.body).await?);

    Ok(Some(data))
}

async fn get_notification_data(
    conn: &MaybeTxConn,
    rconn: &KVObject,
    base_func: impl FnOnce(String) -> NotifyNotifyBase,
    body: &NotificationBody,
) -> ServiceResult<Option<PartsNotifyListEntry>> {
    match body {
        NotificationBody::Followed(follower) => {
            let follower_model =
                try_opt_res!(get_user_profile_by_id(conn, rconn, None, *follower).await?);
            Ok(Some(PartsNotifyListEntry::Followed(NotifyFollowedBody {
                base: base_func("parts/notify/followed".to_owned()),
                data: NotifyFollowedBodyData {
                    follower_nickname: follower_model.basic.nickname,
                    follower_url: format!("/client/user/{}", follower_model.basic.specifier),
                },
            })))
        }
        NotificationBody::FollowRequested(follower) => {
            let follower_model =
                try_opt_res!(get_user_profile_by_id(conn, rconn, None, *follower).await?);
            Ok(Some(PartsNotifyListEntry::FollowRequested(
                NotifyFollowRequestedBody {
                    base: base_func("parts/notify/follow_requested".to_owned()),
                    data: NotifyFollowRequestedBodyData {
                        follower_url: format!("/client/user/{}", follower_model.basic.specifier),
                        follower_nickname: follower_model.basic.nickname,
                    },
                },
            )))
        }
        NotificationBody::Mentioned(author_id, note_id) => {
            let author_model =
                try_opt_res!(get_user_profile_by_id(conn, rconn, None, *author_id).await?);
            Ok(Some(PartsNotifyListEntry::Mentioned(NotifyMentionedBody {
                base: base_func("parts/notify/mentioned".to_owned()),
                data: NotifyMentionedBodyData {
                    author_nickname: author_model.basic.nickname,
                    author_url: format!("/client/user/{}", author_model.basic.specifier),
                    note_url: format!("/client/note/{}", note_id),
                },
            })))
        }
        NotificationBody::Replied(author_id, reply_note_url, replied_note_url) => {
            let author_model =
                try_opt_res!(get_user_profile_by_id(conn, rconn, None, *author_id).await?);
            Ok(Some(PartsNotifyListEntry::Replied(NotifyRepliedBody {
                base: base_func("parts/notify/replied".to_owned()),
                data: NotifyRepliedBodyData {
                    author_url: format!("/client/user/{}", author_model.basic.id),
                    author_nickname: author_model.basic.nickname,
                    reply_note_url: format!("/client/note/{}", reply_note_url),
                    replied_note_url: format!("/client/note/{}", replied_note_url),
                },
            })))
        }
        NotificationBody::Renoted(author_id, renoted_note_id) => {
            let author_model =
                try_opt_res!(get_user_profile_by_id(conn, rconn, None, *author_id).await?);
            Ok(Some(PartsNotifyListEntry::Renoted(NotifyRenotedBody {
                base: base_func("parts/notify/renoted".to_owned()),
                data: NotifyRenotedBodyData {
                    author_url: format!("/client/user/{}", author_model.basic.id),
                    author_nickname: author_model.basic.nickname,
                    renoted_note_url: format!("/client/note/{}", renoted_note_id),
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
