use uuid::Uuid;

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
