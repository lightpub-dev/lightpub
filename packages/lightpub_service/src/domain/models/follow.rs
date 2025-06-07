use chrono::{DateTime, Utc};
use derive_getters::Getters;
use derive_more::Constructor;

use crate::domain::models::user::UserID;

#[derive(Debug, Clone, Copy, PartialEq, Eq, Hash)]
pub struct FollowId(i32);

impl FollowId {
    pub fn as_db(&self) -> i32 {
        self.0
    }

    pub fn from_db_trusted(db: i32) -> Self {
        Self(db)
    }
}

#[derive(Debug, Getters, Constructor)]
pub struct UserFollowEntity {
    id: Option<FollowId>,
    follower_id: UserID,
    followee_id: UserID,
    pending: bool,
    url: Option<String>,
    created_at: DateTime<Utc>,
}

impl UserFollowEntity {
    pub fn create_local(follower_id: UserID, followee_id: UserID, pending: bool) -> Self {
        Self {
            id: None,
            follower_id,
            followee_id,
            pending: pending,
            url: None,
            created_at: Utc::now(),
        }
    }

    pub fn create_remote(
        follower_id: UserID,
        followee_id: UserID,
        pending: bool,
        url: impl Into<String>,
    ) -> Self {
        Self {
            id: None,
            follower_id,
            followee_id,
            pending: pending,
            url: Some(url.into()),
            created_at: Utc::now(),
        }
    }

    pub fn _set_id(&mut self, id: FollowId) {
        self.id = Some(id);
    }

    pub fn confirm(&mut self) {
        self.pending = true;
    }
}
