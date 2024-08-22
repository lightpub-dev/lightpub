use super::{user::UserId, DateTime};

pub struct FollowId(i64);

impl FollowId {
    pub fn from_int(id: i64) -> Self {
        Self(id)
    }
}

// UserFollow entity
pub struct UserFollow {
    id: Option<FollowId>, // id is None when not persisted
    follower: UserId,
    followee: UserId,
    follow_on: DateTime,
}

impl UserFollow {
    pub fn new(follower: UserId, followee: UserId, follow_on: DateTime) -> Self {
        Self {
            id: None,
            follower,
            followee,
            follow_on,
        }
    }
}
