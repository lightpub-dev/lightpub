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

use actix_web::http::header::{CacheControl, CacheDirective};
use actix_web::{get, Responder};

use crate::template::{render_template_builder, PartsTrendData, PartsTrends, Template};
use crate::AppState;
use actix_web::web;
use lightpub_service::services::{timeline::get_trending_hashtags, ServiceResult};

#[get("/trends")]
pub async fn api_get_trends(st: web::Data<AppState>) -> ServiceResult<impl Responder> {
    let trend = get_trending_hashtags(&st.maybe_conn())
        .await?
        .data
        .into_iter()
        .map(|t| PartsTrendData {
            url: t.url,
            hashtag: t.hashtag,
            count: t.count,
        })
        .collect();

    let temp = Template::PartsTrends(PartsTrends { data: trend });

    render_template_builder(st.template(), &temp, |b| {
        b.insert_header(CacheControl(vec![
            CacheDirective::Public,
            CacheDirective::MaxAge(30),
        ]));
    })
}
