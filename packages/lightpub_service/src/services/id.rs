/*
Lightpub: a simple ActivityPub server
Copyright (C) 2025 tinaxd

This program is free software: you can redistribute it and/or modify
it under the terms of the GNU Affero General Public License as published
by the Free Software Foundation, either version 3 of the License, or
(at your option) any later version.

This program is distributed in the hope that it will be useful,
but WITHOUT ANY WARRANTY; without even the implied warranty of
MERCHANTABILITY or FITNESS FOR A PARTICULAR PURPOSE.  See the
GNU Affero General Public License for more details.

You should have received a copy of the GNU Affero General Public License
along with this program.  If not, see <https://www.gnu.org/licenses/>.
*/

use std::{fmt::Display, str::FromStr};

use derive_more::Display;
use serde::{Deserialize, Serialize};
use thiserror::Error;
use ulid::Ulid;
use url::Url;
use uuid::{Uuid, fmt::Simple as UuidSimple};

pub trait Identifier: Display + FromStr {
    fn from_string(s: &str) -> Option<Self> {
        Self::from_str(s).ok()
    }

    fn from_string_trusted(s: &str) -> Self {
        Self::from_string(s).unwrap()
    }

    fn as_local_url(&self, base_url: &Url) -> Url;

    type DBType;
    fn as_db(&self) -> Self::DBType;
    fn from_db_trusted(db: Self::DBType) -> Self;
}

#[derive(Debug, Clone, Serialize, Deserialize, PartialEq, Eq, Display, Hash, Copy)]
pub struct UserID(Ulid);

#[derive(Debug, Clone, Serialize, Deserialize, PartialEq, Eq, Display, Hash, Copy)]
pub struct NoteID(Ulid);

#[derive(Debug, Clone, Serialize, PartialEq, Eq, Display, Hash, Copy)]
pub struct UploadID(UuidSimple);

/// 通知ID
/// 連番なので、DB 内で生成する必要がある。
#[derive(Debug, Clone, Copy, Serialize, Deserialize, PartialEq, Eq, Display, Hash)]
pub struct NotificationID(i32);

impl<'de> Deserialize<'de> for UploadID {
    fn deserialize<D>(deserializer: D) -> Result<Self, D::Error>
    where
        D: serde::Deserializer<'de>,
    {
        let s: uuid::Uuid = Deserialize::deserialize(deserializer)?;
        Ok(Self(UuidSimple::from_uuid(s)))
    }
}

#[derive(Debug, Clone, Error)]
pub enum IdParseError {
    #[error("parse error")]
    ParseError,
}

impl FromStr for UserID {
    type Err = IdParseError;
    fn from_str(s: &str) -> Result<Self, Self::Err> {
        Ulid::from_str(s)
            .map(UserID)
            .map_err(|_| IdParseError::ParseError)
    }
}

impl UserID {
    pub fn new_random() -> Self {
        Self(Ulid::new())
    }
}

impl Identifier for UserID {
    fn as_local_url(&self, base_url: &Url) -> Url {
        base_url
            .join(&format!("/user/{}", self.to_string()))
            .unwrap()
    }

    type DBType = Uuid;
    fn as_db(&self) -> Self::DBType {
        Uuid::from_bytes(self.0.to_bytes())
    }
    fn from_db_trusted(db: Self::DBType) -> Self {
        // check fi
        Self(Ulid::from(db))
    }
}

impl FromStr for NoteID {
    type Err = IdParseError;
    fn from_str(s: &str) -> Result<Self, Self::Err> {
        Ulid::from_str(s)
            .map(NoteID)
            .map_err(|_| IdParseError::ParseError)
    }
}

impl NoteID {
    pub fn new_random() -> Self {
        Self(Ulid::new())
    }
}

impl Identifier for NoteID {
    fn as_local_url(&self, base_url: &Url) -> Url {
        base_url
            .join(&format!("/note/{}", self.to_string()))
            .unwrap()
    }

    type DBType = Uuid;
    fn as_db(&self) -> Uuid {
        Uuid::from_bytes(self.0.to_bytes())
    }
    fn from_db_trusted(db: Self::DBType) -> Self {
        // check fi
        Self(Ulid::from(db))
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
}

impl Identifier for UploadID {
    fn as_local_url(&self, base_url: &Url) -> Url {
        base_url
            .join(&format!("/upload/{}", self.to_string()))
            .unwrap()
    }

    type DBType = Uuid;
    fn as_db(&self) -> Uuid {
        self.0.as_uuid().clone()
    }

    fn from_db_trusted(db: Self::DBType) -> Self {
        Self(db.simple())
    }
}

impl FromStr for NotificationID {
    type Err = IdParseError;
    fn from_str(s: &str) -> Result<Self, Self::Err> {
        let num: i32 = s.parse().map_err(|_| IdParseError::ParseError)?;
        Ok(Self(num))
    }
}

impl Identifier for NotificationID {
    fn as_local_url(&self, base_url: &Url) -> Url {
        base_url
            .join(&format!("/notification/{}", self.to_string()))
            .unwrap()
    }

    type DBType = i32;
    fn from_db_trusted(db: Self::DBType) -> Self {
        Self(db)
    }

    fn as_db(&self) -> Self::DBType {
        self.0
    }
}

#[test]
fn test_note_id_deserialize() {
    use serde_json::json;
    let s = json!("01JKJ32A3V68TRE5EJNEYYV9B6");
    let id: NoteID = serde_json::from_value(s).unwrap();
    assert_eq!(
        id,
        NoteID::from_string_trusted("01JKJ32A3V68TRE5EJNEYYV9B6")
    );
}
