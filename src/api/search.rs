use crate::api::auth::{middleware_auth_jwt_optional, AuthedUser};
use crate::api::note::create_parts_note_from_model;
use actix_web::middleware::from_fn;
use actix_web::{get, web, Responder};
use expected_error::StatusCode;
use lightpub_service::{
    services::{
        create_error_simple,
        search::{search_note_by_content, search_user_by_text},
    },
    ServiceResult,
};
use serde::Deserialize;

use crate::{
    template::{render_template, PartsNotes, PartsUserList, Template},
    AppState,
};

use super::note::create_user_list_from_models;

#[derive(Debug, Clone, Deserialize)]
#[serde(rename_all = "camelCase")]
pub struct SearchQuery {
    q: String,
    search_type: SearchType,
    #[serde(default)]
    suggest: bool,
}

#[derive(Debug, Clone, Deserialize)]
pub enum SearchType {
    #[serde(rename = "user")]
    User,
    #[serde(rename = "note")]
    Note,
}

#[get("/search", wrap = "from_fn(middleware_auth_jwt_optional)")]
pub async fn api_search(
    st: web::Data<AppState>,
    query: web::Query<SearchQuery>,
    auth: web::ReqData<AuthedUser>,
) -> ServiceResult<impl Responder> {
    let allow_remote = !query.suggest;
    let allow_http = st.dev_mode();
    let fuzzy = query.suggest;
    let data = st.request_data();

    match query.search_type {
        SearchType::User => {
            let users = search_user_by_text(
                &st.maybe_conn(),
                &st.rconn(),
                &query.q,
                fuzzy,
                allow_remote,
                allow_http,
                &st.my_domain(),
                &data,
            )
            .await?;
            let users: Vec<_> = users.into_iter().map(|u| (u, None)).collect();
            let temp = Template::PartsUserList(PartsUserList {
                next_url: None, // TODO: pagination for search
                data: create_user_list_from_models(users)?,
            });
            render_template(st.template(), &temp)
        }
        SearchType::Note => {
            let ft = match st.ft() {
                Some(ft) => ft,
                None => {
                    return create_error_simple(
                        StatusCode::BAD_REQUEST,
                        "note search is disabled on this instance",
                    );
                }
            };
            let viewer_id = auth.user_id();
            let notes =
                search_note_by_content(&st.maybe_conn(), &st.rconn(), ft, &query.q, viewer_id)
                    .await?;

            let mut parts = Vec::new();
            for note in notes {
                let part = create_parts_note_from_model(&st, &note, viewer_id, None).await?;
                parts.push(part);
            }
            let temp = Template::PartsNotes(PartsNotes {
                authed: auth.is_authed().into(),
                data: parts,
                next_url: None, // TODO: pagination for search
            });
            render_template(st.template(), &temp)
        }
    }
}
