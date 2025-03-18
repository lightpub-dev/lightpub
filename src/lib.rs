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

use std::{borrow::Cow, fmt::Debug, path::PathBuf, sync::Arc};

use activitypub_federation::config::Data;
use derive_builder::Builder;
use derive_getters::Getters;
use derive_more::Constructor;
use handlebars::{DirectorySourceOptions, Handlebars};
use http_cache_reqwest::{CACacheManager, Cache, CacheMode, HttpCache, HttpCacheOptions};
use lightpub_service::{
    services::{
        db::{Conn, MaybeTxConn},
        fulltext::FTClient,
        kv::KVObject,
        queue::QConn,
    },
    MyFederationData, ServiceState,
};
use reqwest_middleware::{ClientBuilder, ClientWithMiddleware};
use url::Url;

pub mod api;
pub mod client;
pub mod middleware;
pub mod template;

#[derive(Debug, Clone, Constructor)]
pub struct AppState {
    pub(crate) service: ServiceState,
    template: Arc<handlebars::Handlebars<'static>>,
    config: AppConfig,
}

#[derive(Debug, Clone, Builder)]
pub struct AppConfig {
    nodeinfo: NodeInfo,
    #[builder(default)]
    open_registration: bool,
    #[builder(default)]
    report_apub_parse_errors: bool,
}

#[derive(Debug, Clone, Builder, Getters)]
pub struct NodeInfo {
    #[builder(default = "\"A Lightpub instance\".to_string()")]
    name: String,
    #[builder(default)]
    description: String,
}

impl AppState {
    pub fn dev_mode(&self) -> bool {
        self.service.dev_mode()
    }

    pub fn report_apub_parse_errors(&self) -> bool {
        self.config.report_apub_parse_errors
    }

    pub fn nodeinfo(&self) -> &NodeInfo {
        &self.config.nodeinfo
    }

    pub fn is_registration_open(&self) -> bool {
        self.config.open_registration
    }

    pub fn template(&self) -> &handlebars::Handlebars<'static> {
        &self.template
    }

    pub fn template_arc(&self) -> Arc<handlebars::Handlebars<'static>> {
        self.template.clone()
    }

    pub fn maybe_conn(&self) -> MaybeTxConn {
        self.service.maybe_conn()
    }

    pub fn conn(&self) -> &Conn {
        &self.service.conn()
    }

    pub fn rconn(&self) -> KVObject {
        self.service.rconn()
    }

    pub fn qconn(&self) -> &QConn {
        self.service.qconn()
    }

    pub fn ft(&self) -> Option<&FTClient> {
        self.service.ft()
    }

    pub fn request_data(&self) -> Data<MyFederationData> {
        self.service.fed().to_request_data()
    }

    pub fn base_url(&self) -> &Url {
        self.service.base_url()
    }

    pub fn my_domain(&self) -> Cow<str> {
        self.service.my_domain()
    }

    pub fn proxy_client(&self) -> &reqwest_middleware::ClientWithMiddleware {
        self.service.proxy_client()
    }
}

pub fn create_handlebars(dev_mode: bool) -> Arc<Handlebars<'static>> {
    let mut handlebars = Handlebars::new();
    if dev_mode {
        handlebars.set_dev_mode(true);
    }
    handlebars
        .register_templates_directory("./templates", {
            let mut tmp = DirectorySourceOptions::default();
            tmp.tpl_extension = ".html".to_owned();
            tmp
        })
        .unwrap();
    handlebars.register_helper(
        "encodeURIComponent",
        Box::new(client::template::encodeURIComponent),
    );
    let handlebars = Arc::new(handlebars);
    handlebars
}

pub fn create_http_client(dev_mode: bool) -> ClientWithMiddleware {
    let res = reqwest::Client::builder()
        .danger_accept_invalid_certs(dev_mode)
        .build()
        .unwrap();
    ClientBuilder::new(res).build()
}

pub fn create_http_client_with_cache(
    dev_mode: bool,
    base_url: &Url,
    cache_dir: Option<PathBuf>,
) -> ClientWithMiddleware {
    let res = reqwest::Client::builder()
        .danger_accept_invalid_certs(dev_mode)
        .user_agent(format!(
            "{}/{}; +{}",
            env!("CARGO_PKG_NAME"),
            env!("CARGO_PKG_VERSION"),
            base_url
        ))
        .build()
        .unwrap();
    ClientBuilder::new(res)
        .with(Cache(HttpCache {
            mode: CacheMode::Default,
            manager: if let Some(dir) = cache_dir {
                CACacheManager { path: dir.clone() }
            } else {
                CACacheManager::default()
            },
            options: HttpCacheOptions::default(),
        }))
        .build()
}

pub fn registeration_open() -> bool {
    std::env::var("REGISTRATION_OPEN").unwrap_or_else(|_| "false".to_string()) == "true"
}
