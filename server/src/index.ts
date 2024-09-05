import "core-js";
import "reflect-metadata"; // must be the first import
import { registerSqliteServices } from "./registry";

import { Context, Hono } from "hono";

import Ajv, { JSONSchemaType } from "ajv";
import { db } from "./db";
import { sql } from "drizzle-orm";
import { StatusCode } from "hono/utils/http-status";
import { AuthApplicationService } from "./app_service/auth";
import { container } from "tsyringe";

registerSqliteServices();

const app = new Hono();

const ajv = new Ajv();

function errorResponse(c: Context, status: StatusCode, error: string) {
  return c.json({ error }, status);
}

function ise(c: Context, ex: any) {
  console.error(ex);
  return errorResponse(c, 500, "Internal Server Error");
}

app.get("/", (c) => {
  return c.text("Hello Hono!");
});

interface RegisterRequest {
  username: string;
  password: string;
  nickname: string;
}
const registerRequestValidator = ajv.compile({
  type: "object",
  properties: {
    username: { type: "string" },
    password: { type: "string" },
    nickname: { type: "string" },
  },
  required: ["username", "password", "nickname"],
} as JSONSchemaType<RegisterRequest>);
app.post("/auth/register", async (c) => {
  const body = await c.req.json();
  if (!registerRequestValidator(body)) {
    return errorResponse(c, 400, "Invalid request");
  }

  const authService = container.resolve(AuthApplicationService);
  try {
    const result = await authService.register({
      username: body.username,
      password: body.password,
      nickname: body.nickname,
    });
    return c.json(result);
  } catch (ex) {
    return ise(c, ex);
  }
});

interface LoginRequest {
  username: string;
  password: string;
}
const loginRequestValidator = ajv.compile({
  type: "object",
  properties: {
    username: { type: "string" },
    password: { type: "string" },
  },
  required: ["username", "password"],
} as JSONSchemaType<LoginRequest>);
app.post("/auth/login", async (c) => {
  const body = await c.req.json();
  if (!loginRequestValidator(body)) {
    return errorResponse(c, 400, "Invalid request");
  }

  const authService = container.resolve(AuthApplicationService);
  try {
    const result = await authService.login({
      username: body.username,
      password: body.password,
    });
    if (result === null) {
      return errorResponse(c, 401, "Unauthorized");
    }
    return c.json(result);
  } catch (ex) {
    return ise(c, ex);
  }
});

export default app;
