import { migrate } from "drizzle-orm/mysql2/migrator";
import { createSingle } from "./db";

(async () => {
  const { db, connection } = await createSingle();

  // This will run migrations on the database, skipping the ones already applied
  await migrate(db, { migrationsFolder: "./drizzle" });

  // Don't forget to close the connection, otherwise the script will hang
  await connection.end();
})();
