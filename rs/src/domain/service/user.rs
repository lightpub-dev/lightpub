use std::borrow::Cow;

use crate::domain::model::{ctx::AppContext, user::User, URI};

pub struct UserService {}

impl UserService {
    pub fn get_uri<'a>(&self, user: &'a User, ctx: &AppContext) -> Cow<'a, URI> {
        match user.uri() {
            Some(uri) => Cow::Borrowed(uri),
            None => {
                let user_id = user.id().to_string();
                let uri = format!("{}/user/{}", ctx.base_url(), user_id);
                Cow::Owned(URI::from_str(uri).unwrap())
            }
        }
    }

    pub fn get_inbox<'a>(&self, user: &'a User, ctx: &AppContext) -> Cow<'a, URI> {
        match user.inbox() {
            Some(inbox) => Cow::Borrowed(inbox),
            None => {
                let user_id = user.id().to_string();
                let uri = format!("{}/user/{}/inbox", ctx.base_url(), user_id);
                Cow::Owned(URI::from_str(uri).unwrap())
            }
        }
    }

    pub fn get_shared_inbox<'a>(&self, user: &'a User, ctx: &AppContext) -> Cow<'a, URI> {
        match user.shared_inbox() {
            Some(shared_inbox) => Cow::Borrowed(shared_inbox),
            None => {
                let uri = format!("{}/inbox", ctx.base_url());
                Cow::Owned(URI::from_str(uri).unwrap())
            }
        }
    }

    pub fn get_outbox<'a>(&self, user: &'a User, ctx: &AppContext) -> Cow<'a, URI> {
        match user.outbox() {
            Some(outbox) => Cow::Borrowed(outbox),
            None => {
                let user_id = user.id().to_string();
                let uri = format!("{}/user/{}/outbox", ctx.base_url(), user_id);
                Cow::Owned(URI::from_str(uri).unwrap())
            }
        }
    }
}
