use std::{cell::RefCell, fmt::Debug, sync::Arc};
use tokio::sync::Mutex;

use async_trait::async_trait;
use derive_more::Constructor;
use sqlx::{pool::PoolConnection, Decode, Encode, SqliteConnection, SqlitePool, Type};

use crate::{domain::factory::post::PostFactory, holder, Holder};

use super::interface::{
    auth::AuthTokenRepository,
    follow::FollowRepository,
    post::PostRepository,
    uow::{RepositoryManager, UnitOfWork},
    user::UserRepository,
};

pub mod follow;
pub mod post;
pub mod user;

#[derive(Debug)]
pub enum Connection {
    Conn(sqlx::SqliteConnection),
    Tx(Arc<Mutex<TransactionManager>>),
}

pub struct SqliteRepository {
    conn: Connection,
    post_factory: holder!(PostFactory),
}

impl SqliteRepository {
    pub fn new(conn: Connection, post_factory: holder!(PostFactory)) -> Self {
        Self { conn, post_factory }
    }
}

impl Debug for SqliteRepository {
    fn fmt(&self, f: &mut std::fmt::Formatter<'_>) -> std::fmt::Result {
        f.debug_struct("SqliteRepository")
            .field("conn", &self.conn)
            .finish()
    }
}

impl<'c, 'r: 'c> sqlx::Executor<'c> for &'r mut Connection {
    type Database = sqlx::Sqlite;

    fn fetch_many<'e, 'q: 'e, E: 'q>(
        self,
        query: E,
    ) -> futures::stream::BoxStream<
        'e,
        Result<
            sqlx::Either<
                <Self::Database as sqlx::Database>::QueryResult,
                <Self::Database as sqlx::Database>::Row,
            >,
            sqlx::Error,
        >,
    >
    where
        'c: 'e,
        E: sqlx::Execute<'q, Self::Database>,
    {
        match self {
            Connection::Conn(conn) => conn.fetch_many(query),
            Connection::Tx(tx) => {
                let mut tx = tx.lock().await;
                tx.fetch_many(query)
            }
        }
    }

    fn fetch_optional<'e, 'q: 'e, E: 'q>(
        self,
        query: E,
    ) -> futures::future::BoxFuture<
        'e,
        Result<Option<<Self::Database as sqlx::Database>::Row>, sqlx::Error>,
    >
    where
        'c: 'e,
        E: sqlx::Execute<'q, Self::Database>,
    {
        match self {
            Connection::Conn(conn) => conn.fetch_optional(query),
            Connection::Tx(tx) => tx.fetch_optional(query),
        }
    }

    fn prepare_with<'e, 'q: 'e>(
        self,
        sql: &'q str,
        parameters: &'e [<Self::Database as sqlx::Database>::TypeInfo],
    ) -> futures::future::BoxFuture<
        'e,
        Result<<Self::Database as sqlx::database::HasStatement<'q>>::Statement, sqlx::Error>,
    >
    where
        'c: 'e,
    {
        match self {
            Connection::Conn(conn) => conn.prepare_with(sql, parameters),
            Connection::Tx(tx) => tx.prepare_with(sql, parameters),
        }
    }

    fn describe<'e, 'q: 'e>(
        self,
        sql: &'q str,
    ) -> futures::future::BoxFuture<'e, Result<sqlx::Describe<Self::Database>, sqlx::Error>>
    where
        'c: 'e,
    {
        match self {
            Connection::Conn(conn) => conn.describe(sql),
            Connection::Tx(tx) => tx.describe(sql),
        }
    }
}

pub trait IsUuid
where
    Self: Sized,
{
    fn to_uuid(&self) -> uuid::Uuid;
    fn from_uuid(uuid: uuid::Uuid) -> Self;

    fn to_db(&self) -> SqliteUuid {
        SqliteUuid {
            uuid: self.to_uuid(),
        }
    }
    fn from_db(uuid: &SqliteUuid) -> Self {
        Self::from_uuid(uuid.uuid)
    }
}

#[derive(Debug, Clone, Copy, PartialEq, Eq)]
struct SqliteUuid {
    uuid: uuid::Uuid,
}

impl SqliteUuid {
    pub fn into_domain<T: IsUuid>(&self) -> T {
        T::from_uuid(self.uuid)
    }
}

impl Type<sqlx::Sqlite> for SqliteUuid {
    fn compatible(ty: &<sqlx::Sqlite as sqlx::Database>::TypeInfo) -> bool {
        <uuid::fmt::Simple as sqlx::Type<sqlx::Sqlite>>::compatible(ty)
    }

    fn type_info() -> <sqlx::Sqlite as sqlx::Database>::TypeInfo {
        <uuid::fmt::Simple as sqlx::Type<sqlx::Sqlite>>::type_info()
    }
}

impl<'q> Encode<'q, sqlx::Sqlite> for SqliteUuid {
    fn encode_by_ref(
        &self,
        buf: &mut <sqlx::Sqlite as sqlx::database::HasArguments<'q>>::ArgumentBuffer,
    ) -> sqlx::encode::IsNull {
        let simple = self.uuid.simple();
        simple.encode_by_ref(buf)
    }

    fn encode(
        self,
        buf: &mut <sqlx::Sqlite as sqlx::database::HasArguments<'q>>::ArgumentBuffer,
    ) -> sqlx::encode::IsNull
    where
        Self: Sized,
    {
        let simple = self.uuid.simple();
        simple.encode(buf)
    }
}

impl<'r> Decode<'r, sqlx::Sqlite> for SqliteUuid {
    fn decode(
        value: <sqlx::Sqlite as sqlx::database::HasValueRef<'r>>::ValueRef,
    ) -> Result<Self, sqlx::error::BoxDynError> {
        let uuid = <uuid::fmt::Simple as sqlx::Decode<sqlx::Sqlite>>::decode(value)?;
        Ok(SqliteUuid { uuid: uuid.into() })
    }
}

#[derive(Debug)]
pub struct SqliteUow {
    pool: Box<SqlitePool>,
    tx: Option<Arc<Mutex<sqlx::Transaction<'static, sqlx::Sqlite>>>>,
}

impl SqliteUow {
    pub async fn from_pool(conn: SqlitePool) -> Result<Self, anyhow::Error> {
        let conn = Box::new(conn);
        Ok(Self {
            pool: conn,
            tx: None,
        })
    }
}

pub struct TransactionManager {
    tx: Option<sqlx::Transaction<'static, sqlx::Sqlite>>,
}

impl TransactionManager {
    pub async fn commit(&mut self) -> Result<(), sqlx::Error> {
        match self.tx.take() {
            None => panic!("no transaction to commit"),
            Some(t) => t.commit().await,
        }
    }

    pub async fn rollback(&mut self) -> Result<(), sqlx::Error> {
        match self.tx.take() {
            None => panic!("no transaction to rollback"),
            Some(t) => t.rollback().await,
        }
    }

    pub fn tx(&self) -> &sqlx::Transaction<'static, sqlx::Sqlite> {
        self.tx.as_ref().expect("no transaction")
    }

    pub fn new(tx: sqlx::Transaction<'static, sqlx::Sqlite>) -> Self {
        Self { tx: Some(tx) }
    }
}

#[async_trait]
impl UnitOfWork for SqliteUow {
    async fn repository_manager(&mut self) -> Result<holder!(RepositoryManager), anyhow::Error> {
        let tx = self.pool.begin().await?;
        self.tx = Some(Arc::new(Mutex::new(tx)));
    }

    async fn commit(&mut self) -> Result<(), anyhow::Error> {
        // consume current transaction
        if let Some(tx) = self.tx.take() {
            let mut txg = tx.lock().await;
            let txo = txg.as_mut();
        }

        Ok(())
    }
    async fn rollback(&mut self) -> Result<(), anyhow::Error> {
        // consume current transaction
        if let Some(tx) = self.tx.lock().unwrap().take() {
            tx.rollback().await?;
        }

        Ok(())
    }
}

pub struct SqliteRepositoryManager<'c, 'a> {
    conn: &'c mut sqlx::Transaction<'a, sqlx::Sqlite>,
}

impl RepositoryManager for SqliteRepositoryManager<'_, '_> {
    fn user_repository(&self) -> holder!(UserRepository) {
        todo!()
    }

    fn auth_token_repository(&self) -> holder!(AuthTokenRepository) {
        todo!()
    }

    fn follow_repository(&self) -> holder!(FollowRepository) {
        self.make_repository()
    }

    fn post_repository(&self) -> holder!(PostRepository) {
        self.make_repository()
    }
}

impl<'a, 'c> SqliteRepositoryManager<'a, 'c>
where
    'c: 'a,
{
    pub fn new(conn: &'a mut sqlx::Transaction<'c, sqlx::Sqlite>) -> Self {
        Self { conn }
    }

    fn make_repository(&self) -> Holder<SqliteRepository> {
        Holder::new(SqliteRepository::new(Connection::Tx(self.conn), todo!()))
    }
}
