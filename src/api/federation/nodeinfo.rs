use actix_web::{get, web, HttpResponse, Responder};
use lightpub_service::{
    services::{note::count_local_notes, user::get_total_users_count},
    ServiceResult,
};
use serde_json::json;

use crate::AppState;

#[get("/.well-known/nodeinfo")]
pub async fn nodeinfo(st: web::Data<AppState>) -> impl Responder {
    let href = st.base_url().join("/nodeinfo/2.1").unwrap();
    let data = json!({
        "links": [
            {
                "rel": "http://nodeinfo.diaspora.software/ns/schema/2.1",
                "href": href
            }
        ]
    });
    HttpResponse::Ok().json(data)
}

#[get("/nodeinfo/2.1")]
pub async fn nodeinfo_2_1(st: web::Data<AppState>) -> ServiceResult<impl Responder> {
    let total_users_count = get_total_users_count(&st.maybe_conn()).await?;
    let total_notes_count = count_local_notes(&st.maybe_conn()).await?;

    let data = json!({
        "openRegistrations": st.is_registration_open(),
        "protocols": [
            "activitypub"
        ],
        "software": {
            "name": "Lightpub",
            "version": env!("CARGO_PKG_VERSION")
        },
        "usage": {
            "users": {
                "total": total_users_count
            },
            "localPosts": total_notes_count,
        },
        "services": {
            "inbound": [],
            "outbound": []
        },
        "metadata": {
            "nodeName": st.nodeinfo().name(),
            "nodeDescription": st.nodeinfo().description(),
        },
        "version": "2.1"
    });

    Ok(HttpResponse::Ok().json(data))
}
