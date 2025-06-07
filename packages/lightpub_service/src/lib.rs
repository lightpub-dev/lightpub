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

use std::{borrow::Cow, fmt::Debug};

use activitypub_federation::config::FederationConfig;
use derive_more::Constructor;
use services::{
    db::MaybeTxConn, fulltext::FTClient, kv::KVObject, notification::push::WPClient, queue::QConn,
};
use url::Url;

pub mod domain;
pub mod repositories;
pub mod services;
pub mod utils;

pub use services::ServiceResult;

pub type MyFederationData = ServiceStateBase;
pub type MyFederationConfig = FederationConfig<MyFederationData>;

#[derive(Clone, Constructor)]
pub struct ServiceState {
    base: ServiceStateBase,
    fed: MyFederationConfig,
}

impl Debug for ServiceState {
    fn fmt(&self, f: &mut std::fmt::Formatter<'_>) -> std::fmt::Result {
        f.debug_struct("AppState")
            .field("base", &self.base)
            .finish()
    }
}

impl ServiceState {
    pub fn conn(&self) -> &services::db::Conn {
        self.base.conn()
    }

    pub fn rconn(&self) -> KVObject {
        self.base.rconn()
    }

    pub fn qconn(&self) -> &QConn {
        self.base.qconn()
    }

    pub fn base_url(&self) -> &Url {
        self.base.base_url()
    }

    pub fn fed(&self) -> &MyFederationConfig {
        &self.fed
    }

    pub fn maybe_conn(&self) -> MaybeTxConn {
        self.base.maybe_conn()
    }

    pub fn dev_mode(&self) -> bool {
        self.base.dev_mode()
    }

    pub fn my_domain(&self) -> Cow<str> {
        self.base.my_domain()
    }

    pub fn proxy_client(&self) -> &reqwest_middleware::ClientWithMiddleware {
        self.base.proxy_client()
    }

    pub fn ft(&self) -> Option<&FTClient> {
        self.base.ft()
    }

    pub fn wp(&self) -> Option<&WPClient> {
        self.base.wp()
    }
}

#[derive(Clone, Constructor)]
pub struct ServiceStateBase {
    conn: services::db::Conn,
    rconn: KVObject,
    qconn: QConn,

    dev_mode: bool,
    base_url: Url,
    proxy_client: reqwest_middleware::ClientWithMiddleware,
    ft: Option<FTClient>,
    webpush: Option<WPClient>,
}

impl std::fmt::Debug for ServiceStateBase {
    fn fmt(&self, f: &mut std::fmt::Formatter<'_>) -> std::fmt::Result {
        f.debug_struct("ServiceStateBase")
            .field("conn", &self.conn)
            .field("dev_mode", &self.dev_mode)
            .field("base_url", &self.base_url)
            .field("proxy_client", &self.proxy_client)
            .finish_non_exhaustive()
    }
}

impl ServiceStateBase {
    pub fn rconn(&self) -> KVObject {
        self.rconn.clone()
    }

    pub fn conn(&self) -> &services::db::Conn {
        &self.conn
    }

    pub fn qconn(&self) -> &QConn {
        &self.qconn
    }

    pub fn maybe_conn(&self) -> MaybeTxConn {
        MaybeTxConn::Conn(self.conn.clone())
    }

    pub fn dev_mode(&self) -> bool {
        self.dev_mode
    }

    pub fn my_domain(&self) -> Cow<str> {
        Cow::Borrowed(self.base_url.domain().unwrap())
    }

    pub fn base_url(&self) -> &Url {
        &self.base_url
    }

    pub fn proxy_client(&self) -> &reqwest_middleware::ClientWithMiddleware {
        &self.proxy_client
    }

    pub fn ft(&self) -> Option<&FTClient> {
        self.ft.as_ref()
    }

    pub fn wp(&self) -> Option<&WPClient> {
        self.webpush.as_ref()
    }
}
