import "reflect-metadata"; // must be the first import

import { Hono } from "hono";

import { db } from "./db";
import { sql } from "drizzle-orm";

const query = sql`select "hello world" as text`;
const result = db.get<{ text: string }>(query);
console.log(result);

const app = new Hono();

app.get("/", (c) => {
  return c.text("Hello Hono!");
});

export default app;
