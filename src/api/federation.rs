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

use activitypub_federation::actix_web::signing_actor;
use activitypub_federation::config::Data;
use activitypub_federation::{
    actix_web::inbox::receive_activity,
    fetch::webfinger::{build_webfinger_response, extract_webfinger_name},
    traits::ActivityHandler,
};
use actix_web::{get, post, web, HttpRequest, HttpResponse, Responder};
use lightpub_service::MyFederationData;
use serde::{Deserialize, Serialize};
use tracing::{debug, warn};
use url::Url;

use lightpub_service::services::apub::{
    report_apub_error, AcceptActivity, AnnounceActivity, CreateActivity, DeleteActivity,
    LikeActivity, RejectActivity, UndoActivity, UpdateActivity,
};
use lightpub_service::services::FederationServiceError;
use lightpub_service::services::{
    apub::FollowActivity,
    id::Identifier,
    user::{get_user_by_spec, UserSpecifier, UserWithApubModel},
    ServiceError, ServiceResult,
};

pub mod nodeinfo;

use crate::AppState;

#[derive(Deserialize)]
pub struct WebfingerQuery {
    resource: String,
}

#[get("/.well-known/webfinger")]
pub async fn webfinger(
    query: web::Query<WebfingerQuery>,
    st: web::Data<AppState>,
) -> ServiceResult<HttpResponse> {
    let data = st.request_data();
    let name = extract_webfinger_name(&query.resource, &data);
    let name = match name {
        Ok(name) => name,
        Err(e) => {
            debug!("Failed to extract webfinger name: {:?}", e);
            match e {
                activitypub_federation::error::Error::WebfingerResolveFailed(_) => {
                    return Ok(HttpResponse::NotFound().finish());
                }
                _ => return Err(ServiceError::unknown(e)),
            }
        }
    };
    let db_user = get_user_by_spec(
        &st.maybe_conn(),
        &st.rconn(),
        &UserSpecifier::local_username(name),
        &st.my_domain(),
    )
    .await?;

    match db_user {
        None => {
            return Ok(HttpResponse::NotFound().finish());
        }
        Some(user) => Ok(HttpResponse::Ok().json(build_webfinger_response(
            query.resource.clone(),
            user.id().as_local_url(&st.base_url()),
        ))),
    }
}

#[derive(Debug, Clone, Deserialize, Serialize)]
#[serde(untagged)]
#[enum_delegate::implement(ActivityHandler)]
pub enum SharedInboxActivity {
    Follow(FollowActivity),
    Create(CreateActivity),
    Undo(UndoActivity),
    Reject(RejectActivity),
    Accept(AcceptActivity),
    Like(LikeActivity),
    Delete(DeleteActivity),
    Announce(AnnounceActivity),
    Update(UpdateActivity),
}

#[derive(Debug, Clone, Deserialize, Serialize)]
#[serde(untagged)]
#[enum_delegate::implement(ActivityHandler)]
pub enum UserInboxActivity {
    Follow(FollowActivity),
    Create(CreateActivity),
    Undo(UndoActivity),
    Reject(RejectActivity),
    Accept(AcceptActivity),
    Like(LikeActivity),
    Delete(DeleteActivity),
    Announce(AnnounceActivity),
    Update(UpdateActivity),
}

#[post("/inbox")]
pub async fn api_shared_inbox(
    st: web::Data<AppState>,
    req: HttpRequest,
    body: web::Bytes,
) -> ServiceResult<impl Responder> {
    handle_inbox(st, req, body, false).await
}

pub(crate) async fn handle_inbox(
    st: web::Data<AppState>,
    req: HttpRequest,
    body: web::Bytes,
    is_user: bool,
) -> ServiceResult<impl Responder> {
    let data = st.request_data();
    debug!("incoming inbox: {body:?}");
    let body_copy = body.clone();
    let result = if is_user {
        receive_activity::<UserInboxActivity, UserWithApubModel, _>(req, body, &data).await
    } else {
        receive_activity::<SharedInboxActivity, UserWithApubModel, _>(req, body, &data).await
    };

    match result {
        Ok(res) => Ok(res),
        Err(FederationServiceError::FederationError(fe)) => {
            use activitypub_federation::error::Error;
            match fe {
                Error::ParseReceivedActivity(e, url) => {
                    warn!("Failed to parse activity from {url:?}: {e:#?}");
                    if st.report_apub_parse_errors() {
                        let body_str = String::from_utf8(body_copy.to_vec());
                        match body_str {
                            Ok(body) => {
                                report_apub_error(
                                    &st.maybe_conn(),
                                    body,
                                    format!("Failed to parse activity from {url:?}: {e:#?}"),
                                )
                                .await?
                            }
                            Err(_) => {
                                warn!("Failed to parse body as utf8");
                            }
                        }
                    }
                    Ok(HttpResponse::BadRequest().finish())
                }
                _ => Err(fe.into()),
            }
        }
        r => r.map_err(|e| e.into()),
    }
}

pub async fn apub_auth(
    req: &HttpRequest,
    body: web::Bytes,
    data: &Data<MyFederationData>,
) -> ServiceResult<Option<UserWithApubModel>> {
    let actor = signing_actor::<UserWithApubModel>(req, Some(body), data).await;
    debug!("federation auth actor: {:?}", actor);
    let actor = match actor {
        Ok(a) => Some(a),
        Err(FederationServiceError::FederationError(e)) => {
            use activitypub_federation::error::Error;
            match e {
                Error::ActivitySignatureInvalid | Error::ActivityBodyDigestInvalid => None,
                _ => return Err(e.into()),
            }
        }
        Err(e) => return Err(e.into()),
    };
    Ok(actor)
}
