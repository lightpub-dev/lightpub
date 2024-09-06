import "core-js";
import "reflect-metadata"; // must be the first import
import { registerMysqlServices } from "./registry";

import { Context, Hono } from "hono";
import { createMiddleware } from "hono/factory";

import Ajv, { JSONSchemaType } from "ajv";
import { StatusCode } from "hono/utils/http-status";
import { AuthApplicationService } from "./app_service/auth";
import { container } from "tsyringe";
import { posts, secrets, users } from "./mysql_schema";
import { LightpubException } from "./error";
import { createDB } from "./db";
import { UserApplicationService } from "./app_service/user";
import { parseUserspec } from "./utils/user";
import { FollowApplicationService } from "./app_service/follow";
import { PaginatedResponse, parseLimit } from "./utils/pagination";

registerMysqlServices();

const DEFAULT_LIMIT = 20;

const app = new Hono();

const ajv = new Ajv();

function errorResponse(c: Context, status: StatusCode, error: string) {
  return c.json({ error }, status);
}

function ise(c: Context, ex: any) {
  console.error(ex);
  return errorResponse(c, 500, "Internal Server Error");
}

app.onError((err, c) => {
  console.error(err);
  if (err instanceof LightpubException) {
    return errorResponse(c, err.status as StatusCode, err.message);
  }
  return ise(c, err);
});

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

  const result = await authService.register({
    username: body.username,
    password: body.password,
    nickname: body.nickname,
  });
  return c.json({
    user_id: result.userId,
  });
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

  const result = await authService.login({
    username: body.username,
    password: body.password,
  });
  if (result === null) {
    return errorResponse(c, 401, "Unauthorized");
  }
  return c.json({
    token: result.token,
  });
});

const USER_ID = "userId" as const;
function authMiddlewareBuilder<T extends true | false>(opts: { force: T }) {
  return createMiddleware<{
    Variables: {
      [USER_ID]: string | (T extends false ? null : never);
    };
  }>(async (c, next) => {
    const token = c.req.header("Authorization");
    if (token === undefined) {
      // no auth header
      if (opts.force) {
        return errorResponse(c, 401, "Unauthorized");
      }
      return next();
    }

    if (!token.startsWith("Bearer ")) {
      return errorResponse(c, 400, "Invalid Authorization header");
    }

    const authService = container.resolve(AuthApplicationService);
    const result = await authService.verifyToken(token.slice(7));
    if (!result.success) {
      return errorResponse(c, 401, "Unauthorized");
    }

    c.set("userId", result.userId);
    await next();
  });
}

const requireAuthMiddleware = authMiddlewareBuilder({ force: true });
const optionalAuthMiddleware = authMiddlewareBuilder({ force: false });

app.put("/user/:user_spec/follow", requireAuthMiddleware, async (c) => {
  const userSpec = c.req.param("user_spec");
  const userService = container.resolve(UserApplicationService);

  const follower = c.get(USER_ID);

  const userId = await userService.findUserId(parseUserspec(userSpec));
  if (userId === null) {
    return errorResponse(c, 404, "User not found");
  }

  const followService = container.resolve(FollowApplicationService);
  await followService.follow(follower, userId.userId);
  return c.json({ message: "OK" });
});

app.delete("/user/:user_spec/follow", requireAuthMiddleware, async (c) => {
  const userSpec = c.req.param("user_spec");
  const userService = container.resolve(UserApplicationService);

  const follower = c.get(USER_ID);

  const userId = await userService.findUserId(parseUserspec(userSpec));
  if (userId === null) {
    return errorResponse(c, 404, "User not found");
  }

  const followService = container.resolve(FollowApplicationService);
  await followService.unfollow(follower, userId.userId);
  return c.json({ message: "OK" });
});

app.get("/user/:user_spec/followings", optionalAuthMiddleware, async (c) => {
  const userSpec = c.req.param("user_spec");
  const before = c.req.query("before");
  const limit = parseLimit(c.req.query("limit"), DEFAULT_LIMIT);
  const userService = container.resolve(UserApplicationService);

  const userId = await userService.findUserId(parseUserspec(userSpec));
  if (userId === null) {
    return errorResponse(c, 404, "User not found");
  }

  const followService = container.resolve(FollowApplicationService);
  const users = await followService.getFollowings(userId.userId, {
    limit: limit,
    before: before === undefined ? undefined : new Date(before),
  });
  return c.json(
    new PaginatedResponse(limit, users, (u) => u.followAt).response()
  );
});

app.get("/user/:user_spec/followers", optionalAuthMiddleware, async (c) => {
  const userSpec = c.req.param("user_spec");
  const before = c.req.query("before");
  const limit = parseLimit(c.req.query("limit"), DEFAULT_LIMIT);
  const userService = container.resolve(UserApplicationService);

  const userId = await userService.findUserId(parseUserspec(userSpec));
  if (userId === null) {
    return errorResponse(c, 404, "User not found");
  }

  const followService = container.resolve(FollowApplicationService);
  const users = await followService.getFollowers(userId.userId, {
    limit: limit,
    before: before === undefined ? undefined : new Date(before),
  });
  return c.json(
    new PaginatedResponse(limit, users, (u) => u.followAt).response()
  );
});

app.post("/debug/truncate", async (c) => {
  const db = await createDB();
  await db.transaction(async (tx) => {
    await tx.delete(secrets).execute();
    await tx.delete(posts).execute();
    await tx.delete(users).execute();
  });
  return c.text("OK");
});

export default app;
