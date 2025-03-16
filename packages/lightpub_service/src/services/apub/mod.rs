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

use actix_web::http::StatusCode;
use expected_error_derive::ExpectedError;
use itertools::Itertools;
use once_cell::sync::Lazy;
use thiserror::Error;
use url::Url;

mod accept;
mod announce;
mod create;
mod delete;
mod follow;
mod like;
mod reject;
mod reporter;
mod undo;
mod update;

use super::{ServiceError, ServiceResult};

pub use accept::{AcceptActivity, AcceptableObject};
pub use announce::{AnnounceActivity, AnnounceableObject};
pub use create::{CreatableObject, CreateActivity};
pub use delete::DeleteActivity;
pub use follow::FollowActivity;
pub use like::{LikeActivity, LikeableObject};
pub use reject::{RejectActivity, RejectableObject};
pub use reporter::report_apub_error;
pub use undo::{UndoActivity, UndoableObject};
pub use update::{UpdatableObject, UpdateActivity};

#[derive(Debug, Clone, Error, ExpectedError)]
pub enum ApubError {
    #[error("activity actor and object actor do not match")]
    #[ee(status(StatusCode::FORBIDDEN))]
    ActorMismatch,
}

fn actor_check(a1: &Url, a2: &Url) -> ServiceResult<()> {
    if a1 != a2 {
        return Err(ServiceError::known(ApubError::ActorMismatch));
    }
    Ok(())
}

const PUBLIC_URL_: &'static str = "https://www.w3.org/ns/activitystreams#Public";

pub static PUBLIC_URL: Lazy<Url> = Lazy::new(|| Url::parse(PUBLIC_URL_).unwrap());

pub static PUBLIC_URLS: Lazy<Vec<String>> = Lazy::new(|| {
    vec![
        PUBLIC_URL_.to_string(),
        "Public".to_string(),
        "as:Public".to_string(),
    ]
});

pub fn contains_public_url(urls: &[Url]) -> bool {
    PUBLIC_URLS
        .iter()
        .any(|pub_url| urls.iter().map(|u| u.as_str()).contains(pub_url.as_str()))
}
