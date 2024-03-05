use uuid::{fmt::Simple, Uuid};

type URL = String;

#[derive(Debug, Clone, PartialEq)]
pub enum UserSpecifier {
    ID(Uuid),
    Username(String, Option<String>),
    URL(URL),
}

#[derive(Debug, Clone)]
pub enum UserSpecifierParseError {
    InvalidFormat,
    InvalidLocalID,
}

impl UserSpecifier {
    pub fn parse(s: &str) -> Result<Self, UserSpecifierParseError> {
        if s.starts_with("@") {
            let parts: Vec<&str> = s[1..].split("@").collect();
            if parts.len() == 1 {
                // no hostname
                return Ok(UserSpecifier::Username(parts[0].to_string(), None));
            }
            if parts.len() == 2 {
                return Ok(UserSpecifier::Username(
                    parts[0].to_string(),
                    Some(parts[1].to_string()),
                ));
            }
            return Err(UserSpecifierParseError::InvalidFormat);
        }

        // try to parse a uuid
        if let Ok(u) = uuid::Uuid::parse_str(s) {
            return Ok(UserSpecifier::ID(u));
        }

        Err(UserSpecifierParseError::InvalidFormat)
    }

    pub fn from_id(id: impl Into<Uuid>) -> Self {
        UserSpecifier::ID(id.into())
    }
}
