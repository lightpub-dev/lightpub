use crate::services::id::Identifier;
use crate::services::id::UserID;
use url::Url;

#[derive(Debug, Clone)]
pub enum UserSpecifier {
    ID(UserID),
    Username(String, Option<String>),
    Specifier(String),
    URL(Url),
}

impl UserSpecifier {
    pub fn from_str(spec: &str, my_domain: &str) -> Option<Self> {
        if !spec.starts_with("@") {
            return Some(UserSpecifier::ID(UserID::from_string(&spec)?));
        }

        let at_split: Vec<&str> = spec.split('@').collect();

        let (username, domain) = match at_split.len() {
            2 => (at_split[1].to_string(), None),
            3 => {
                if at_split[2] == my_domain {
                    (at_split[1].to_string(), None)
                } else {
                    (at_split[1].to_string(), Some(at_split[2].to_string()))
                }
            }
            _ => return None,
        };

        Some(UserSpecifier::Username(username, domain))
    }

    pub fn local_username(username: impl Into<String>) -> Self {
        UserSpecifier::Username(username.into(), None)
    }

    pub fn username_and_domain(username: impl Into<String>, domain: impl Into<String>) -> Self {
        let domain = domain.into();
        let domain = if domain == "" { None } else { Some(domain) };
        Self::username_and_domain_opt(username, domain)
    }

    pub fn username_and_domain_opt(
        username: impl Into<String>,
        domain: Option<impl Into<String>>,
    ) -> Self {
        let domain = domain.map(|s| s.into());
        UserSpecifier::Username(username.into(), domain)
    }

    pub fn url(url: Url) -> Self {
        UserSpecifier::URL(url)
    }

    /// UserSpecifier::Specifier の場合は UserSpecifier::Username または UserSpecifier::ID に変換する
    /// 変換に失敗した場合は None を返す
    pub fn try_parse_specifier(self, my_domain: &str) -> Option<Self> {
        match self {
            UserSpecifier::Specifier(spec) => Self::from_str(&spec, my_domain),
            v => Some(v),
        }
    }

    pub fn omit_my_domain(self, my_domain: &str) -> Self {
        match self {
            Self::Username(username, Some(domain)) if domain == my_domain => {
                UserSpecifier::Username(username, None)
            }
            v => v,
        }
    }
}

impl std::fmt::Display for UserSpecifier {
    fn fmt(&self, f: &mut std::fmt::Formatter<'_>) -> std::fmt::Result {
        match self {
            UserSpecifier::ID(id) => id.fmt(f),
            UserSpecifier::Specifier(s) => s.fmt(f),
            UserSpecifier::URL(url) => url.fmt(f),
            UserSpecifier::Username(username, domain) => {
                if let Some(domain) = domain {
                    write!(f, "@{}@{}", username, domain)
                } else {
                    write!(f, "@{}", username)
                }
            }
        }
    }
}
