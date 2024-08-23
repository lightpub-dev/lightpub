use sqlx::Executor;

pub mod follow;

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

impl<'c> sqlx::Executor<'c> for &mut SqliteRepository<'_> {
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
        match self.conn {
            Connection::Conn(mut conn) => conn.fetch_many(query),
            Connection::Tx(mut tx) => tx.fetch_many(query),
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
        match self.conn {
            Connection::Conn(mut conn) => conn.fetch_optional(query),
            Connection::Tx(mut tx) => tx.fetch_optional(query),
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
        match self.conn {
            Connection::Conn(mut conn) => conn.prepare_with(sql, parameters),
            Connection::Tx(mut tx) => tx.prepare_with(sql, parameters),
        }
    }

    fn describe<'e, 'q: 'e>(
        self,
        sql: &'q str,
    ) -> futures::future::BoxFuture<'e, Result<sqlx::Describe<Self::Database>, sqlx::Error>>
    where
        'c: 'e,
    {
        match self.conn {
            Connection::Conn(mut conn) => conn.describe(sql),
            Connection::Tx(mut tx) => tx.describe(sql),
        }
    }
}
