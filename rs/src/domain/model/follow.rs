use super::user::UserId;

// UserFollow value object
pub struct UserFollow {
    follower: UserId,
    followee: UserId,
}
