use crate::model::apub::Activity;
use rsa::RsaPrivateKey;
use serde::{Deserialize, Serialize};

use super::ApubSigner;

#[derive(Serialize, Deserialize, Debug, Clone)]
pub enum WorkerTask {
    PostToInbox(PostToInboxPayload),
}

#[derive(Serialize, Deserialize, Debug, Clone)]
pub struct PostToInboxPayload {
    pub url: String,
    pub activity: Activity,
    pub actor: SignerPayload,
}

#[derive(Serialize, Deserialize, Debug, Clone)]
pub struct SignerPayload {
    pub user_id: String,
    pub private_key: RsaPrivateKey,
    pub private_key_id: String,
}

impl ApubSigner for &SignerPayload {
    fn get_private_key(&self) -> RsaPrivateKey {
        self.private_key.clone()
    }

    fn get_private_key_id(&self) -> String {
        self.private_key_id.clone()
    }

    fn get_user_id(&self) -> String {
        self.user_id.clone()
    }
}
