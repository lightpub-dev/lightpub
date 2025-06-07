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
pub struct NoteID(Ulid);

/// 通知ID
/// 連番なので、DB 内で生成する必要がある。
#[derive(Debug, Clone, Copy, Serialize, Deserialize, PartialEq, Eq, Display, Hash)]
pub struct NotificationID(i32);

impl Identifier for UserID {
    fn as_local_url(&self, base_url: &Url) -> Url {
        base_url
            .join(&format!("/user/{}", self.to_string()))
            .unwrap()
    }

    type DBType = Vec<u8>;
    fn as_db(&self) -> Self::DBType {
        self.0.to_bytes().to_vec()
    }
    fn from_db_trusted(db: Self::DBType) -> Self {
        // check fi
        Self(Ulid::from_bytes(slice_to_bytes(&db)))
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
