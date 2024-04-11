use lightpub_config::Config;
use lightpub_model::HasRemoteUri;

#[derive(Debug, Clone)]
pub struct IDGetterService {
    config: Config,
}

#[derive(Debug, Clone, Copy, PartialEq, Eq)]
pub enum UserAttribute {
    Inbox,
    Outbox,
    Following,
    Followers,
    Liked,
    PublicKey,
}

impl UserAttribute {
    pub fn as_url(&self) -> String {
        match self {
            Self::Inbox => "/inbox",
            Self::Outbox => "/outbox",
            Self::Following => "/following",
            Self::Followers => "/followers",
            Self::Liked => "/liked",
            Self::PublicKey => "#main-key",
        }
        .to_string()
    }
}

#[derive(Debug, Clone, Copy, PartialEq, Eq)]
pub enum PostAttribute {
    CreateActivity,
}

impl PostAttribute {
    pub fn as_url(&self) -> String {
        match self {
            Self::CreateActivity => "/activity",
        }
        .to_string()
    }
}

impl IDGetterService {
    pub fn new(config: Config) -> Self {
        Self { config }
    }

    pub fn is_our_host(&self, host: &str) -> bool {
        host == self.config.hostname
    }

    pub fn get_user_id(&self, user: &impl HasRemoteUri) -> String {
        if let Some(uri) = user.get_remote_uri() {
            uri
        } else {
            format!("{}/user/{}", self.config.base_url(), user.get_local_id())
        }
    }

    pub fn get_user_id_attr(
        &self,
        user: &impl HasRemoteUri,
        attr: UserAttribute,
    ) -> Option<String> {
        let base = match user.get_remote_uri() {
            Some(uri) => {
                if uri.starts_with(&self.config.base_url()) {
                    self.get_user_id(user)
                } else {
                    return None;
                }
            }
            None => self.get_user_id(user),
        };

        Some(format!("{}{}", base, attr.as_url()))
    }

    pub fn get_post_id(&self, post: &impl HasRemoteUri) -> String {
        if let Some(uri) = post.get_remote_uri() {
            uri
        } else {
            format!("{}/post/{}", self.config.base_url(), post.get_local_id())
        }
    }

    pub fn get_post_id_attr(
        &self,
        post: &impl HasRemoteUri,
        attr: PostAttribute,
    ) -> Option<String> {
        match post.get_remote_uri() {
            Some(_) => None,
            None => {
                let base = self.get_post_id(post);
                format!("{}{}", base, attr.as_url()).into()
            }
        }
    }

    pub fn get_reaction_id(&self, internal_reaction_id: &str, with_emoji: bool) -> String {
        format!(
            "{}/{}/{}",
            self.config.base_url(),
            if with_emoji { "reaction" } else { "favorite" },
            internal_reaction_id
        )
    }

    pub fn get_follower_request_id(&self, follow_req: &impl HasRemoteUri) -> String {
        if let Some(uri) = follow_req.get_remote_uri() {
            uri
        } else {
            format!(
                "{}/follow-req/{}",
                self.config.base_url(),
                follow_req.get_local_id()
            )
        }
    }

    fn extract_id_from_uri(&self, uri: &str, resource_type: &str) -> Option<String> {
        let base = format!("{}/{}/", self.config.base_url(), resource_type);
        if uri.starts_with(&self.config.base_url()) && uri.starts_with(&base) {
            Some(uri[base.len()..].to_string())
        } else {
            None
        }
    }

    pub fn extract_follow_request_id(&self, follow_req_uri: &str) -> Option<String> {
        self.extract_id_from_uri(follow_req_uri, "follow-req")
    }

    pub fn extract_local_user_id(&self, user_uri: &str) -> Option<String> {
        self.extract_id_from_uri(user_uri, "user")
    }

    pub fn extract_local_post_id(&self, post_uri: &str) -> Option<String> {
        self.extract_id_from_uri(post_uri, "post")
    }
}
