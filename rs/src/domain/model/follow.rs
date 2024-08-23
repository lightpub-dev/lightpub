use super::{user::UserId, DateTime};

#[derive(Debug, Clone, PartialEq, Eq, sqlx::Type)]
#[sqlx(transparent)]
pub struct FollowId(i64);

impl FollowId {
    pub fn from_int(id: i64) -> Self {
        Self(id)
    }
}

// UserFollow entity
#[derive(Debug)]
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

    pub fn follower(&self) -> &UserId {
        &self.follower
    }

    pub fn followee(&self) -> &UserId {
        &self.followee
    }

    pub fn follow_on(&self) -> &DateTime {
        &self.follow_on
    }

    pub fn id(&self) -> Option<&FollowId> {
        self.id.as_ref()
    }

    pub fn set_id(&mut self, id: FollowId) {
        self.id = Some(id);
    }
}

impl PartialEq for UserFollow {
    fn eq(&self, other: &Self) -> bool {
        match (&self.id, &other.id) {
            (Some(self_id), Some(other_id)) => *self_id == *other_id,
            _ => false,
        }
    }
}
