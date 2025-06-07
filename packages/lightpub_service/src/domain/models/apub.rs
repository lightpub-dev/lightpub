#[derive(Debug, Clone, PartialEq, Eq)]
pub struct ActorID(String);

impl ActorID {
    pub fn new(url: impl Into<String>) -> Self {
        Self(url.into())
    }

    pub fn as_str(&self) -> &str {
        &self.0
    }

    pub fn into_inner(self) -> String {
        self.0
    }
}

#[derive(Debug, Clone, PartialEq, Eq)]
pub struct ApubPrivateKey(String);

#[derive(Debug, Clone, PartialEq, Eq)]
pub struct ApubPublicKey(String);
