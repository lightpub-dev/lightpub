use crate::apub::Activity;
use rsa::RsaPrivateKey;
use serde::{Deserialize, Serialize};

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
