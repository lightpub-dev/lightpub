use std::borrow::Cow;

use crate::config::Config;

#[derive(Debug)]
pub struct IDGetterService<'a> {
    config: Cow<'a, Config>,
}

pub trait HasRemoteUri {
    fn get_local_id(&self) -> String;
    fn get_remote_uri(&self) -> Option<String>;
}

impl<'a> IDGetterService<'a> {
    pub fn new(config: Cow<'a, Config>) -> Self {
        Self { config }
    }

    pub fn get_user_id(&self, user: impl HasRemoteUri) -> String {
        if let Some(uri) = user.get_remote_uri() {
            uri
        } else {
            format!("{}/user/{}", self.config.base_url(), user.get_local_id())
        }
    }

    pub fn get_post_id(&self, post: impl HasRemoteUri) -> String {
        if let Some(uri) = post.get_remote_uri() {
            uri
        } else {
            format!("{}/post/{}", self.config.base_url(), post.get_local_id())
        }
    }

    pub fn get_follower_request_id(&self, follow_req: impl HasRemoteUri) -> String {
        if let Some(uri) = follow_req.get_remote_uri() {
            uri
        } else {
            format!(
                "{}/follow-req/{}",
                self.config.base_url(),
                follow_req.get_local_id()
            )
        }
    }
}
