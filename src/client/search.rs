use actix_web::{get, web, Responder};

use crate::{
    template::{render_template, Template},
    AppState,
};

#[get("/search")]
pub async fn client_get_search(st: web::Data<AppState>) -> impl Responder {
    render_template(st.template(), &Template::Search(()))
}
