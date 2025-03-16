use actix_web::{get, web, HttpResponse, Responder};
use lightpub_service::{services::search::search_user_by_text, ServiceResult};
use serde::Deserialize;

use crate::{
    template::{render_template, PartsUserList, Template},
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

#[get("/search")]
pub async fn api_search(
    st: web::Data<AppState>,
    query: web::Query<SearchQuery>,
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
        _ => Ok(HttpResponse::Ok().json("not implemented")),
    }
}
