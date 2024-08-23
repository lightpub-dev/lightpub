use sqlx::{Decode, Encode, Type};

use crate::domain::model::user::UserId;

use super::IsUuid;

impl Type<sqlx::Sqlite> for UserId {
    fn type_info() -> <sqlx::Sqlite as sqlx::Database>::TypeInfo {
        <uuid::fmt::Simple as sqlx::Type<sqlx::Sqlite>>::type_info()
    }

    fn compatible(ty: &<sqlx::Sqlite as sqlx::Database>::TypeInfo) -> bool {
        <uuid::fmt::Simple as sqlx::Type<sqlx::Sqlite>>::compatible(ty)
    }
}

impl IsUuid for UserId {
    fn to_uuid(&self) -> uuid::Uuid {
        self.id()
    }

    fn from_uuid(uuid: uuid::Uuid) -> Self {
        Self::from_uuid(uuid)
    }
}
