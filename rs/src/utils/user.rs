use serde::Deserialize;
use uuid::Uuid;

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

    pub fn from_username(username: String, hostname: Option<String>) -> Self {
        UserSpecifier::Username(username, hostname)
    }
}

impl<T: Into<Uuid>> From<T> for UserSpecifier {
    fn from(id: T) -> Self {
        UserSpecifier::ID(id.into())
    }
}

impl<'de> Deserialize<'de> for UserSpecifier {
    fn deserialize<D>(deserializer: D) -> Result<Self, D::Error>
    where
        D: serde::Deserializer<'de>,
    {
        deserializer.deserialize_str(UserSpecifierVisitor {})
    }
}

struct UserSpecifierVisitor {}

impl<'de> serde::de::Visitor<'de> for UserSpecifierVisitor {
    type Value = UserSpecifier;

    fn expecting(&self, formatter: &mut std::fmt::Formatter) -> std::fmt::Result {
        formatter.write_str("a user specifier")
    }

    fn visit_str<E>(self, value: &str) -> Result<UserSpecifier, E>
    where
        E: serde::de::Error,
    {
        UserSpecifier::parse(value).map_err(|e| E::custom(format!("{:?}", e)))
    }
}
