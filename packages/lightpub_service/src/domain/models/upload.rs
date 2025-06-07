use std::str::FromStr;

use super::{IdParseError, slice_to_bytes};
use derive_more::Display;
use serde::{Deserialize, Serialize};
use uuid::{Uuid, fmt::Simple as UuidSimple};

#[derive(Debug, Clone, Serialize, PartialEq, Eq, Display, Hash, Copy)]
pub struct UploadID(UuidSimple);

impl<'de> Deserialize<'de> for UploadID {
    fn deserialize<D>(deserializer: D) -> Result<Self, D::Error>
    where
        D: serde::Deserializer<'de>,
    {
        let s: uuid::Uuid = Deserialize::deserialize(deserializer)?;
        Ok(Self(UuidSimple::from_uuid(s)))
    }
}

impl FromStr for UploadID {
    type Err = IdParseError;
    fn from_str(s: &str) -> Result<Self, Self::Err> {
        uuid::Uuid::parse_str(s)
            .map(|u| UploadID(u.simple()))
            .map_err(|_| IdParseError::ParseError)
    }
}

impl UploadID {
    pub fn new_random() -> Self {
        Self(uuid::Uuid::new_v4().simple())
    }

    pub fn as_db(&self) -> Vec<u8> {
        self.0.as_uuid().as_bytes().to_vec()
    }

    pub fn from_db_trusted(db: Vec<u8>) -> Self {
        Self(Uuid::from_bytes(slice_to_bytes(&db)).simple())
    }
}
