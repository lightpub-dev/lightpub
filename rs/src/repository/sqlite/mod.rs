use sqlx::{Decode, Encode, Type};

pub mod follow;
pub mod user;

#[derive(Debug)]
pub enum Connection<'a> {
    Conn(sqlx::SqliteConnection),
    Tx(sqlx::Transaction<'a, sqlx::Sqlite>),
}

#[derive(Debug)]
pub struct SqliteRepository<'a> {
    conn: Connection<'a>,
}

impl<'a> SqliteRepository<'a> {
    pub fn new(conn: Connection<'a>) -> Self {
        Self { conn }
    }
}

impl<'c, 'r: 'c> sqlx::Executor<'c> for &'r mut SqliteRepository<'_> {
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
        match &mut self.conn {
            Connection::Conn(conn) => conn.fetch_many(query),
            Connection::Tx(tx) => tx.fetch_many(query),
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
        match &mut self.conn {
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
        match &mut self.conn {
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
        match &mut self.conn {
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
