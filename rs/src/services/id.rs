use std::borrow::Cow;

use crate::{config::Config, models::HasRemoteUri};

#[derive(Debug)]
pub struct IDGetterService<'a> {
    config: Cow<'a, Config>,
}

#[derive(Debug, Clone, Copy, PartialEq, Eq)]
pub enum UserAttribute {
    Inbox,
    Outbox,
    Following,
    Followers,
    Liked,
}

impl UserAttribute {
    pub fn to_string(&self) -> String {
        match self {
            Self::Inbox => "inbox",
            Self::Outbox => "outbox",
            Self::Following => "following",
            Self::Followers => "followers",
            Self::Liked => "liked",
        }
        .to_string()
    }
}

impl<'a> IDGetterService<'a> {
    pub fn new(config: Cow<'a, Config>) -> Self {
        Self { config }
    }

    pub fn get_user_id(&self, user: &impl HasRemoteUri) -> String {
        if let Some(uri) = user.get_remote_uri() {
            uri
        } else {
            format!("{}/user/{}", self.config.base_url(), user.get_local_id())
        }
    }

    pub fn get_user_id_attr(
        &self,
        user: &impl HasRemoteUri,
        attr: UserAttribute,
    ) -> Option<String> {
        match user.get_remote_uri() {
            Some(_) => None,
            None => {
                let base = self.get_user_id(user);
                Some(format!("{}/{}", base, attr.to_string()))
            }
        }
    }

    pub fn get_post_id(&self, post: &impl HasRemoteUri) -> String {
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
