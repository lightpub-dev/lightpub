use ammonia::Builder;
use serde::Serialize;

#[derive(Debug, Clone, PartialEq, Eq, PartialOrd, Ord, Hash, Serialize)]
#[serde(transparent)]
pub struct CleanString(String);

impl std::fmt::Display for CleanString {
    fn fmt(&self, f: &mut std::fmt::Formatter<'_>) -> std::fmt::Result {
        write!(f, "{}", self.0)
    }
}

impl CleanString {
    pub fn inner(&self) -> &str {
        &self.0
    }

    pub fn into_inner(self) -> String {
        self.0
    }

    pub fn clean(s: &str) -> CleanString {
        let cleaned = Builder::default()
            .rm_tags(["img"]) // disallow remote imgs
            .clean(s)
            .to_string();
        CleanString(cleaned)
    }

    pub fn clean_text(s: &str) -> CleanString {
        CleanString(ammonia::clean_text(s))
    }

    pub fn already_cleaned_dangerous(s: impl Into<String>) -> CleanString {
        CleanString(s.into())
    }
}
