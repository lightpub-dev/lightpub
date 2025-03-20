use actix_web::{post, web, HttpResponse, Responder};
use chrono::{DateTime, Utc};
use expected_error::StatusCode;
use lightpub_service::services::create_error_simple;
use lightpub_service::services::note::rebuild_note_fulltext_index;
use lightpub_service::services::user::is_admin;
use serde::Deserialize;

use super::auth::AuthedUser;
use crate::api::auth::middleware_auth_jwt_required;
use crate::AppState;
use actix_web::middleware::from_fn;

#[derive(Debug, Clone, Deserialize)]
#[serde(rename_all = "camelCase")]
pub struct AdminApiRebuildNoteFTRequest {
    after_date: Option<DateTime<Utc>>,
}

#[post(
    "/admin/ft/rebuild/note",
    wrap = "from_fn(middleware_auth_jwt_required)"
)]
pub async fn admin_api_rebuild_note_fulltext(
    st: web::Data<AppState>,
    params: web::Json<AdminApiRebuildNoteFTRequest>,
    auth: web::ReqData<AuthedUser>,
) -> impl Responder {
    let user_id = auth.user_id_unwrap();
    if !is_admin(&st.maybe_conn(), user_id).await? {
        return create_error_simple(StatusCode::FORBIDDEN, "You are not an admin");
    }

    if let Some(ft) = st.ft() {
        let after_date = params.after_date;
        rebuild_note_fulltext_index(&st.maybe_conn(), ft, after_date).await?;
        return Ok(HttpResponse::NoContent().finish());
    } else {
        create_error_simple(
            StatusCode::INTERNAL_SERVER_ERROR,
            "Fulltext service is not available",
        )
    }
}
