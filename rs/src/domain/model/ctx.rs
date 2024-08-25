use super::URI;

pub struct AppContext {
    base_url: URI,
}

impl AppContext {
    pub fn new(base_url: URI) -> Self {
        Self { base_url }
    }

    pub fn base_url(&self) -> &URI {
        &self.base_url
    }
}
