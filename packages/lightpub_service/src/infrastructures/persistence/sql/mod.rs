use redis::AsyncCommands;
use sea_orm::DatabaseConnection;
use serde::{Serialize, de::DeserializeOwned};
use tracing::error;

use crate::{ServiceResult, services::MapToUnknown};

pub mod follow;
pub mod user;

#[derive(Debug, Clone)]
pub struct LpDbConn {
    db: DatabaseConnection,
}

impl LpDbConn {
    pub fn db(&self) -> &DatabaseConnection {
        &self.db
    }
}

#[derive(Clone)]
pub struct LpKvConn {
    conn: redis::aio::ConnectionManager,
}

impl std::fmt::Debug for LpKvConn {
    fn fmt(&self, f: &mut std::fmt::Formatter<'_>) -> std::fmt::Result {
        f.debug_struct("LpKvConn").finish_non_exhaustive()
    }
}

impl LpKvConn {
    async fn get_raw(&self, key: &str) -> ServiceResult<Option<Vec<u8>>> {
        let mut c = self.conn.clone();
        c.get(key).await.map_err_unknown()
    }

    async fn set_raw(
        &self,
        key: &str,
        value: &[u8],
        ttl: Option<std::time::Duration>,
    ) -> ServiceResult<()> {
        let mut c = self.conn.clone();
        if let Some(ttl) = ttl {
            c.set_ex(key, value, ttl.as_secs()).await.map_err_unknown()
        } else {
            c.set(key, value).await.map_err_unknown()
        }
    }

    async fn delete_(&self, key: &str) -> ServiceResult<()> {
        let mut c = self.conn.clone();
        c.del(key).await.map_err_unknown()
    }

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
