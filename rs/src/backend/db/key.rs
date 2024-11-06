use async_trait::async_trait;
use derive_more::Constructor;
use gen_span::gen_span;
use rsa::{pkcs8::DecodePublicKey, RsaPublicKey};
use sqlx::SqlitePool;
use uuid::fmt::Simple;

use crate::backend::AllUserFinderService;
use crate::holder;
use crate::model::UserSpecifier;
use crate::utils::key::{KeyFetcher, KeyFetcherResult};

#[derive(Constructor)]
pub struct DBKeyFetcher {
    pool: SqlitePool,
    finder: holder!(AllUserFinderService),
}

#[derive(Debug)]
struct KeyRow {
    #[allow(dead_code)]
    id: String,
    owner_id: Simple,
    public_key: String,
}

#[gen_span]
#[async_trait]
impl KeyFetcher for DBKeyFetcher {
    async fn fetch_pubkey(&mut self, id: &str) -> Result<KeyFetcherResult, anyhow::Error> {
        let key_row = sqlx::query_as!(
            KeyRow,
            r#"
            SELECT id, owner_id AS `owner_id: Simple`, public_key
            FROM user_keys
            WHERE id = ?
            "#,
            id
        )
        .fetch_optional(&self.pool)
        .await?;

        if let Some(key_row) = key_row {
            return Ok(KeyFetcherResult {
                key: RsaPublicKey::from_public_key_pem(&key_row.public_key)?,
                user_id: key_row.owner_id,
            });
        }

        // if key not found, try to find user by id
        self.finder
            .find_user_by_specifier(&UserSpecifier::from_url(id))
            .await?;
        // if user found, key should be stored in user_keys table

        let key_row = sqlx::query_as!(
            KeyRow,
            r#"
                SELECT id, owner_id AS `owner_id: Simple`, public_key
                FROM user_keys
                WHERE id = ?
                "#,
            id
        )
        .fetch_optional(&self.pool)
        .await?;

        if let Some(key_row) = key_row {
            return Ok(KeyFetcherResult {
                key: RsaPublicKey::from_public_key_pem(&key_row.public_key)?,
                user_id: key_row.owner_id,
            });
        }

        Err(anyhow::anyhow!("Key not found in db"))
    }
}
