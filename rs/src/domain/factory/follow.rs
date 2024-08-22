use crate::domain::model::{follow::UserFollow, user::UserId, DateTime};

pub trait UserFollowFactory {
    fn create(&mut self, follower_id: UserId, followee_id: UserId) -> UserFollow;
}

pub struct DefaultUserFollowFactory {}

impl UserFollowFactory for DefaultUserFollowFactory {
    fn create(&mut self, follower_id: UserId, followee_id: UserId) -> UserFollow {
        UserFollow::new(follower_id, followee_id, DateTime::now())
    }
}

impl DefaultUserFollowFactory {
    pub fn new() -> Self {
        Self {}
    }
}
