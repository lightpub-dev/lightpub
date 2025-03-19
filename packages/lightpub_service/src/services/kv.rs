use async_trait::async_trait;
use serde::{Serialize, de::DeserializeOwned};
use std::sync::Arc;
use tracing::error;

use super::{MapToUnknown, ServiceResult};

pub type KVObject = Arc<dyn KV + Send + Sync>;

#[async_trait]
pub trait KV {
    async fn get_raw(&self, key: &str) -> ServiceResult<Option<Vec<u8>>>;
    async fn set_raw(
        &self,
        key: &str,
        value: &[u8],
        ttl: Option<std::time::Duration>,
    ) -> ServiceResult<()>;
    async fn delete_(&self, key: &str) -> ServiceResult<()>;
}

impl dyn KV + Send + Sync {
    pub async fn get<V: DeserializeOwned>(&self, key: impl AsRef<str>) -> ServiceResult<Option<V>> {
        let key = key.as_ref();
        let bytes = self.get_raw(key).await?;
        match bytes {
            None => Ok(None),
            Some(bytes) => {
                let value = serde_json::from_slice(&bytes);
                match value {
                    Ok(value) => Ok(Some(value)),
                    Err(e) => {
                        // デシリアライズに失敗した場合は, そのキーを削除して None を返す
                        error!("Failed to deserialize kv value (key={key}): {e}");
                        self.delete_(key).await?;
                        Ok(None)
                    }
                }
            }
        }
    }

    pub async fn set<V: Serialize>(&self, key: impl AsRef<str>, value: &V) -> ServiceResult<()> {
        let key = key.as_ref();
        let value = serde_json::to_vec(value).map_err_unknown()?;
        self.set_raw(key, &value, None).await
    }

    pub async fn set_ttl<V: Serialize>(
        &self,
        key: impl AsRef<str>,
        value: &V,
        ttl: std::time::Duration,
    ) -> ServiceResult<()> {
        let key = key.as_ref();
        let value = serde_json::to_vec(value).map_err_unknown()?;
        self.set_raw(key, &value, Some(ttl)).await
    }

    pub async fn delete(&self, key: impl AsRef<str>) -> ServiceResult<()> {
        self.delete_(key.as_ref()).await
    }
}
