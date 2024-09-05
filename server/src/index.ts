import "reflect-metadata"; // must be the first import

import { Hono } from "hono";

import { db } from "./db";
import { sql } from "drizzle-orm";

const app = new Hono();

app.get("/", (c) => {
  return c.text("Hello Hono!");
});

export default app;
