use serde::Serialize;
use uuid::Uuid;

// PostId value object
#[derive(Debug, PartialEq, Eq, Clone)]
pub struct PostId(Uuid);

impl PostId {
    pub fn from_uuid(uuid: Uuid) -> Self {
        Self(uuid)
    }

    pub fn from_str(uuid: &str) -> Option<Self> {
        Uuid::parse_str(uuid).map(Self).ok()
    }
}

#[derive(Debug, Clone)]
pub enum PostSpecifier {
    ID(Uuid),
    URI(String),
}

impl PostSpecifier {
    pub fn from_id(id: impl Into<Uuid>) -> Self {
        PostSpecifier::ID(id.into())
    }

    pub fn from_uri(uri: impl Into<String>) -> Self {
        PostSpecifier::URI(uri.into())
    }
}

impl From<Uuid> for PostSpecifier {
    fn from(id: Uuid) -> Self {
        PostSpecifier::ID(id)
    }
}
