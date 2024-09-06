import { drizzle, MySql2Database } from "drizzle-orm/mysql2/driver";
import { createPool, createConnection, Connection, Pool } from "mysql2/promise";

let singleConnection: Awaited<ReturnType<typeof createSingle>> | null = null;
let poolConnection: Awaited<ReturnType<typeof createPoolConnection>> | null =
  null;

export async function createSingle(): Promise<{
  db: MySql2Database<Record<string, never>>;
  connection: Connection;
}> {
  if (singleConnection !== null) return singleConnection;

  const connection = await createConnection({
    host: process.env["DB_HOST"] ?? "127.0.0.1",
    port: Number.parseInt(process.env["DB_PORT"] ?? "3306"),
    user: process.env["DB_USER"] ?? "lightpub",
    password: process.env["DB_PASSWORD"] ?? "lightpub",
    database: process.env["DB_DATABASE"] ?? "lightpub",
  });
  const db = drizzle(connection);
  const conn = {
    db,
    connection,
  };
  singleConnection = conn;
  return conn;
}

export async function createPoolConnection(): Promise<{
  db: MySql2Database<Record<string, never>>;
  connection: Pool;
}> {
  if (poolConnection !== null) return poolConnection;

  const connection = await createPool({
    host: process.env["DB_HOST"] ?? "127.0.0.1",
    port: Number.parseInt(process.env["DB_PORT"] ?? "3306"),
    user: process.env["DB_USER"] ?? "lightpub",
    password: process.env["DB_PASSWORD"] ?? "lightpub",
    database: process.env["DB_DATABASE"] ?? "lightpub",
  });
  const db = drizzle(connection);
  const conn = {
    db,
    connection,
  };
  poolConnection = conn;
  return conn;
}

export async function createDB(): Promise<
  MySql2Database<Record<string, never>>
> {
  const { db } = await createSingle();
  return db;
}
