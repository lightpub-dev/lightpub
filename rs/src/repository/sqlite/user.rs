use crate::domain::model::user::UserId;

use super::IsUuid;

impl IsUuid for UserId {
    fn to_uuid(&self) -> uuid::Uuid {
        self.id()
    }

    fn from_uuid(uuid: uuid::Uuid) -> Self {
        Self::from_uuid(uuid)
    }
}
