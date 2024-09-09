import "core-js";
import "reflect-metadata"; // must be the first import
import { registerMysqlServices } from "./registry";

import { Context, Hono } from "hono";
import { createMiddleware } from "hono/factory";

import Ajv, { JSONSchemaType } from "ajv";
import { StatusCode } from "hono/utils/http-status";
import { AuthApplicationService } from "./app_service/auth";
import { container } from "tsyringe";
import { posts, secrets, userFollows, users } from "./mysql_schema";
import { LightpubException } from "./error";
import { createDB } from "./db";
import { UserApplicationService } from "./app_service/user";
import { parseUserspec } from "./utils/user";
import { FollowApplicationService } from "./app_service/follow";
import { PaginatedResponse, parseLimit } from "./utils/pagination";
import {
  PostCreateApplicationService,
  PostFetchApplicationService,
  PostReactionApplicationService,
} from "./app_service/post";
import { sql } from "drizzle-orm";

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

interface PostCreateRequest {
  content?: string;
  privacy: "public" | "unlisted" | "follower" | "private";
  reply_to_id?: string;
  repost_of_id?: string;
}
const postCreateRequestValidator = ajv.compile({
  type: "object",
  properties: {
    content: { type: "string", nullable: true },
    privacy: {
      type: "string",
      enum: ["public", "unlisted", "follower", "private"],
    },
    reply_to_id: { type: "string", nullable: true },
    repost_of_id: { type: "string", nullable: true },
  },
  required: ["privacy"],
} as JSONSchemaType<PostCreateRequest>);
app.post("/post", requireAuthMiddleware, async (c) => {
  const body = await c.req.json();
  if (!postCreateRequestValidator(body)) {
    return errorResponse(c, 400, "Invalid request");
  }

  const postCreateService = container.resolve(PostCreateApplicationService);
  let result: Awaited<ReturnType<typeof postCreateService.createPost>>;
  if (body.content === undefined && body.repost_of_id !== undefined) {
    // repost
    result = await postCreateService.createPost({
      authorId: c.get(USER_ID),
      privacy: body.privacy,
      repostOfId: body.repost_of_id,
    });
  } else if (
    body.content !== undefined &&
    body.reply_to_id !== undefined &&
    body.repost_of_id === undefined
  ) {
    // reply
    result = await postCreateService.createPost({
      authorId: c.get(USER_ID),
      content: body.content,
      privacy: body.privacy,
      replyToId: body.reply_to_id,
    });
  } else if (
    body.content !== undefined &&
    body.reply_to_id === undefined &&
    body.repost_of_id !== undefined
  ) {
    // repost
    result = await postCreateService.createPost({
      authorId: c.get(USER_ID),
      content: body.content,
      privacy: body.privacy,
      repostOfId: body.repost_of_id,
    });
  } else if (
    body.content !== undefined &&
    body.reply_to_id === undefined &&
    body.repost_of_id === undefined
  ) {
    // normal post
    result = await postCreateService.createPost({
      authorId: c.get(USER_ID),
      content: body.content,
      privacy: body.privacy,
    });
  } else {
    throw new LightpubException(400, "Invalid request");
  }

  return c.json({
    post_id: result.id,
  });
});

app.delete("/post/:post_id", requireAuthMiddleware, async (c) => {
  const postId = c.req.param("post_id");
  const postCreateService = container.resolve(PostCreateApplicationService);
  await postCreateService.deletePost(postId, c.get(USER_ID));
  return c.json({ message: "OK" });
});

app.get("/post/:post_id", optionalAuthMiddleware, async (c) => {
  const postId = c.req.param("post_id");
  const postCreateService = container.resolve(PostFetchApplicationService);
  const post = await postCreateService.fetchPost(postId, c.get(USER_ID));
  if (post === null) {
    throw new LightpubException(404, "Post not found");
  }

  return c.json({
    id: post.id,
    url: post.url,
    author: {
      id: post.authorId,
    },
    content: post.content,
    privacy: post.privacy,
    reply_to_id: post.replyToId,
    repost_of_id: post.repostOfId,
    created_at: post.createdAt.toISOString(),
  });
});

app.put("/post/:post_id/favorite", requireAuthMiddleware, async (c) => {
  const postId = c.req.param("post_id");
  const postCreateService = container.resolve(PostReactionApplicationService);
  await postCreateService.favoritePost(c.get(USER_ID), postId);
  return c.json({ message: "OK" });
});

app.delete("/post/:post_id/favorite", requireAuthMiddleware, async (c) => {
  const postId = c.req.param("post_id");
  const postCreateService = container.resolve(PostReactionApplicationService);
  await postCreateService.deleteFavorite(c.get(USER_ID), postId);
  return c.json({ message: "OK" });
});

app.put("/post/:post_id/bookmark", requireAuthMiddleware, async (c) => {
  const postId = c.req.param("post_id");
  const postCreateService = container.resolve(PostReactionApplicationService);
  await postCreateService.bookmarkPost(c.get(USER_ID), postId);
  return c.json({ message: "OK" });
});

app.delete("/post/:post_id/bookmark", requireAuthMiddleware, async (c) => {
  const postId = c.req.param("post_id");
  const postCreateService = container.resolve(PostReactionApplicationService);
  await postCreateService.deleteBookmark(c.get(USER_ID), postId);
  return c.json({ message: "OK" });
});

interface ReactionRequest {
  reaction: string;
  add: boolean;
}
const reactionRequestValidator = ajv.compile({
  type: "object",
  properties: {
    reaction: { type: "string" },
    add: { type: "boolean" },
  },
  required: ["reaction", "add"],
} as JSONSchemaType<ReactionRequest>);
app.post("/post/:post_id/reaction", requireAuthMiddleware, async (c) => {
  const postId = c.req.param("post_id");
  const body = await c.req.json();
  if (!reactionRequestValidator(body)) {
    return errorResponse(c, 400, "Invalid request");
  }

  const postCreateService = container.resolve(PostReactionApplicationService);
  if (body.add) {
    await postCreateService.reactionPost(c.get(USER_ID), postId, body.reaction);
  } else {
    await postCreateService.deleteReaction(
      c.get(USER_ID),
      postId,
      body.reaction
    );
  }
  return c.json({ message: "OK" });
});

if (process.env.NODE_ENV === "development") {
  app.post("/debug/truncate", async (c) => {
    const db = await createDB();
    await db.transaction(async (tx) => {
      await tx.execute(sql`SET foreign_key_checks = 0`);
      await tx.delete(secrets).execute();
      await tx.delete(userFollows).execute();
      await tx.delete(posts).execute();
      await tx.delete(users).execute();
      await tx.execute(sql`SET foreign_key_checks = 1`);
    });
    return c.text("OK");
  });
}

export default app;
