use uuid::Uuid;

use crate::domain::model::{post::PostId, user::UserId};

pub trait IDGenerationService {
    fn generate_post_id(&mut self) -> PostId;
    fn generate_user_id(&mut self) -> UserId;
}

pub struct V7Generator {}

impl IDGenerationService for V7Generator {
    fn generate_post_id(&mut self) -> PostId {
        PostId::from_uuid(Uuid::now_v7())
    }

    fn generate_user_id(&mut self) -> UserId {
        UserId::from_uuid(Uuid::now_v7())
    }
}
