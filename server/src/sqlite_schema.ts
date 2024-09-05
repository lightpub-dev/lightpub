import {
  sqliteTable,
  text,
  integer,
  AnySQLiteColumn,
} from "drizzle-orm/sqlite-core";

export const users = sqliteTable("users", {
  id: text("id").notNull().primaryKey(),
  username: text("username", {
    length: 128,
  }).notNull(),
  hostname: text("hostname", {
    length: 512,
  }),
  password: text("password", {
    length: 256,
  }),
  nickname: text("nickname", {
    length: 128,
  }).notNull(),
  bio: text("bio", {
    length: 2048,
  })
    .notNull()
    .default(""),
  url: text("url", {
    length: 512,
  }),
  privateKey: text("privateKey", {
    length: 1024,
  }),
  publicKey: text("publicKey", {
    length: 1024,
  }),
  createdAt: integer("createdAt").notNull(),
  deletedAt: integer("deletedAt"),
});

export const posts = sqliteTable("posts", {
  id: text("id").notNull().primaryKey(),
  url: text("url", {
    length: 512,
  }),
  authorId: text("authorId")
    .notNull()
    .references(() => users.id),
  privacy: text("privacy", {
    enum: ["public", "unlisted", "follower", "private"],
  }),
  replyToId: text("replyToId").references((): AnySQLiteColumn => posts.id),
  repostOfId: text("repostOfId").references((): AnySQLiteColumn => posts.id),
  createdAt: integer("createdAt").notNull(),
  deletedAt: integer("deletedAt"),
});

export const userFollows = sqliteTable("userFollows", {
  id: integer("id").notNull().primaryKey(),
  followerId: text("followerId")
    .notNull()
    .references(() => users.id),
  followeeId: text("followeeId")
    .notNull()
    .references(() => users.id),
  followAt: integer("followAt").notNull(),
});
