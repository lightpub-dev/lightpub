import { defineConfig } from "drizzle-kit";

export default defineConfig({
  schema: "./src/mysql_schema.ts",
  out: "./drizzle",
  dialect: "mysql", // 'postgresql' | 'mysql' | 'sqlite'
  dbCredentials: {
    host: process.env["DB_HOST"] ?? "127.0.0.1",
    port: Number.parseInt(process.env["DB_PORT"] ?? "3306"),
    user: process.env["DB_USER"] ?? "lightpub",
    password: process.env["DB_PASSWORD"] ?? "lightpub",
    database: process.env["DB_DATABASE"] ?? "lightpub",
  },
});
