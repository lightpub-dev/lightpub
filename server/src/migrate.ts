import { db, connection } from "./db";
import { migrate } from "drizzle-orm/bun-sqlite/migrator";

(async () => {
  // This will run migrations on the database, skipping the ones already applied
  migrate(db, { migrationsFolder: "./drizzle" });

  // Don't forget to close the connection, otherwise the script will hang
  connection.close();
})();
