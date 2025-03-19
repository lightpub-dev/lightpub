/*
Lightpub: a simple ActivityPub server
Copyright (C) 2025 tinaxd

This program is free software: you can redistribute it and/or modify
it under the terms of the GNU Affero General Public License as published
by the Free Software Foundation, either version 3 of the License, or
(at your option) any later version.

This program is distributed in the hope that it will be useful,
but WITHOUT ANY WARRANTY; without even the implied warranty of
MERCHANTABILITY or FITNESS FOR A PARTICULAR PURPOSE.  See the
GNU Affero General Public License for more details.

You should have received a copy of the GNU Affero General Public License
along with this program.  If not, see <https://www.gnu.org/licenses/>.
*/

use async_trait::async_trait;
use redis::AsyncCommands;
use sea_orm::{
    AccessMode, ConnectOptions, ConnectionTrait, Database, DatabaseConnection, DatabaseTransaction,
    DbErr, ExecResult, IsolationLevel, QueryResult, SqlErr, Statement, TransactionTrait,
};

use super::{MapToUnknown, ServiceResult, kv::KV};

#[derive(Debug, Clone)]
pub struct Conn {
    db: DatabaseConnection,
}

impl Conn {
    pub fn db(&self) -> &DatabaseConnection {
        &self.db
    }

    pub async fn as_tx(&self) -> ServiceResult<TxConn> {
        Ok(TxConn {
            conn: self.clone(),
            db_tx: self.db.begin().await.map_err_unknown()?,
        })
    }

    pub async fn as_tx_with_config(
        &self,
        isolation: impl Into<Option<IsolationLevel>>,
        mode: impl Into<Option<AccessMode>>,
    ) -> ServiceResult<TxConn> {
        Ok(TxConn {
            conn: self.clone(),
            db_tx: self
                .db
                .begin_with_config(isolation.into(), mode.into())
                .await
                .map_err_unknown()?,
        })
    }
}

impl Conn {
    pub async fn create(connect_option: impl Into<ConnectOptions>) -> Self {
        let db = Database::connect(connect_option).await.unwrap();
        Conn { db }
    }
}

#[derive(Debug)]
pub struct TxConn {
    conn: Conn,
    db_tx: DatabaseTransaction,
}

impl TxConn {
    pub fn tx(&self) -> &DatabaseTransaction {
        &self.db_tx
    }

    pub async fn commit(self) -> ServiceResult<Conn> {
        self.db_tx.commit().await.map_err_unknown()?;
        Ok(self.conn)
    }

    pub async fn rollback(self) -> ServiceResult<()> {
        self.db_tx.rollback().await.map_err_unknown()
    }
}

impl From<Conn> for MaybeTxConn {
    fn from(conn: Conn) -> Self {
        MaybeTxConn::Conn(conn)
    }
}

impl From<TxConn> for MaybeTxConn {
    fn from(tx_conn: TxConn) -> Self {
        MaybeTxConn::TxConn(tx_conn)
    }
}

#[derive(Debug)]
pub enum MaybeTxConn {
    Conn(Conn),
    TxConn(TxConn),
}

impl MaybeTxConn {
    pub async fn commit(self) -> ServiceResult<Conn> {
        match self {
            MaybeTxConn::Conn(c) => Ok(c),
            MaybeTxConn::TxConn(tx) => tx.commit().await,
        }
    }

    pub async fn rollback(self) -> ServiceResult<()> {
        match self {
            MaybeTxConn::Conn(_) => Ok(()),
            MaybeTxConn::TxConn(tx) => tx.rollback().await,
        }
    }
}

impl ConnectionTrait for MaybeTxConn {
    fn get_database_backend(&self) -> sea_orm::DbBackend {
        match self {
            MaybeTxConn::Conn(c) => c.db.get_database_backend(),
            MaybeTxConn::TxConn(t) => t.db_tx.get_database_backend(),
        }
    }

    fn execute<'life0, 'async_trait>(
        &'life0 self,
        stmt: sea_orm::Statement,
    ) -> ::core::pin::Pin<
        Box<
            dyn ::core::future::Future<Output = Result<sea_orm::ExecResult, DbErr>>
                + ::core::marker::Send
                + 'async_trait,
        >,
    >
    where
        'life0: 'async_trait,
        Self: 'async_trait,
    {
        match self {
            MaybeTxConn::Conn(c) => c.db.execute(stmt),
            MaybeTxConn::TxConn(t) => t.db_tx.execute(stmt),
        }
    }

    fn execute_unprepared<'life0, 'life1, 'async_trait>(
        &'life0 self,
        sql: &'life1 str,
    ) -> ::core::pin::Pin<
        Box<
            dyn ::core::future::Future<Output = Result<ExecResult, DbErr>>
                + ::core::marker::Send
                + 'async_trait,
        >,
    >
    where
        'life0: 'async_trait,
        'life1: 'async_trait,
        Self: 'async_trait,
    {
        match self {
            MaybeTxConn::Conn(c) => c.db.execute_unprepared(sql),
            MaybeTxConn::TxConn(t) => t.db_tx.execute_unprepared(sql),
        }
    }

    fn query_one<'life0, 'async_trait>(
        &'life0 self,
        stmt: Statement,
    ) -> ::core::pin::Pin<
        Box<
            dyn ::core::future::Future<Output = Result<Option<QueryResult>, DbErr>>
                + ::core::marker::Send
                + 'async_trait,
        >,
    >
    where
        'life0: 'async_trait,
        Self: 'async_trait,
    {
        match self {
            MaybeTxConn::Conn(c) => c.db.query_one(stmt),
            MaybeTxConn::TxConn(t) => t.db_tx.query_one(stmt),
        }
    }

    fn query_all<'life0, 'async_trait>(
        &'life0 self,
        stmt: Statement,
    ) -> ::core::pin::Pin<
        Box<
            dyn ::core::future::Future<Output = Result<Vec<QueryResult>, DbErr>>
                + ::core::marker::Send
                + 'async_trait,
        >,
    >
    where
        'life0: 'async_trait,
        Self: 'async_trait,
    {
        match self {
            MaybeTxConn::Conn(c) => c.db.query_all(stmt),
            MaybeTxConn::TxConn(t) => t.db_tx.query_all(stmt),
        }
    }
}

#[derive(Clone)]
pub struct RedisConn {
    cm: redis::aio::ConnectionManager,
}

impl RedisConn {
    pub fn new(cm: redis::aio::ConnectionManager) -> Self {
        Self { cm }
    }
}

impl std::fmt::Debug for RedisConn {
    fn fmt(&self, f: &mut std::fmt::Formatter<'_>) -> std::fmt::Result {
        f.debug_struct("RedisConn").finish_non_exhaustive()
    }
}

#[async_trait]
impl KV for RedisConn {
    async fn get_raw(&self, key: &str) -> ServiceResult<Option<Vec<u8>>> {
        let mut c = self.cm.clone();
        c.get(key).await.map_err_unknown()
    }

    async fn set_raw(
        &self,
        key: &str,
        value: &[u8],
        ttl: Option<std::time::Duration>,
    ) -> ServiceResult<()> {
        let mut c = self.cm.clone();
        if let Some(ttl) = ttl {
            c.set_ex(key, value, ttl.as_secs()).await.map_err_unknown()
        } else {
            c.set(key, value).await.map_err_unknown()
        }
    }

    async fn delete_(&self, key: &str) -> ServiceResult<()> {
        let mut c = self.cm.clone();
        c.del(key).await.map_err_unknown()
    }
}

/// Dummy KV store that does nothing.
/// Always return None for get.
#[derive(Debug, Clone, Default)]
pub struct DummyKV {}

#[async_trait]
impl KV for DummyKV {
    async fn get_raw(&self, _: &str) -> ServiceResult<Option<Vec<u8>>> {
        Ok(None)
    }

    async fn set_raw(
        &self,
        _: &str,
        _: &[u8],
        _: Option<std::time::Duration>,
    ) -> ServiceResult<()> {
        Ok(())
    }

    async fn delete_(&self, _: &str) -> ServiceResult<()> {
        Ok(())
    }
}

pub fn is_unique_constraint_error(err: &DbErr) -> bool {
    matches!(err.sql_err(), Some(SqlErr::UniqueConstraintViolation(_)))
}
