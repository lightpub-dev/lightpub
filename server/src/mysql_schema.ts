import {
  AnyMySqlColumn,
  bigint,
  char,
  datetime,
  mysqlEnum,
  mysqlTable,
  serial,
  text,
  unique,
  varchar,
} from "drizzle-orm/mysql-core";

export const users = mysqlTable(
  "users",
  {
    id: char("id", {
      length: 32,
    })
      .notNull()
      .primaryKey(),
    username: varchar("username", {
      length: 64,
    }).notNull(),
    hostname: varchar("hostname", {
      length: 128,
    }),
    password: varchar("password", {
      length: 60,
    }),
    nickname: varchar("nickname", {
      length: 255,
    }).notNull(),
    bio: text("bio").notNull().default(""),
    url: varchar("url", {
      length: 512,
    }),
    privateKey: varchar("privateKey", {
      length: 1024,
    }),
    publicKey: varchar("publicKey", {
      length: 1024,
    }),
    createdAt: datetime("createdAt", {
      fsp: 6,
    }).notNull(),
    deletedAt: datetime("deletedAt", {
      fsp: 6,
    }),
  },
  (t) => ({
    usernameUnique: unique().on(t.username, t.hostname),
  })
);

export const secrets = mysqlTable("secrets", {
  key: varchar("key", { length: 64 }).notNull().primaryKey(),
  value: text("value").notNull(),
});

export const posts = mysqlTable("posts", {
  id: char("id", {
    length: 32,
  })
    .notNull()
    .primaryKey(),
  url: varchar("url", {
    length: 512,
  }),
  authorId: char("authorId", {
    length: 32,
  })
    .notNull()
    .references(() => users.id),
  content: text("content"),
  privacy: mysqlEnum("privacy", [
    "public",
    "unlisted",
    "follower",
    "private",
  ]).notNull(),
  replyToId: char("replyToId", {
    length: 32,
  }).references((): AnyMySqlColumn => posts.id),
  repostOfId: char("repostOfId", {
    length: 32,
  }).references((): AnyMySqlColumn => posts.id),
  createdAt: datetime("createdAt", {
    fsp: 6,
  }).notNull(),
  deletedAt: datetime("deletedAt", {
    fsp: 6,
  }),
});

export const userFollows = mysqlTable(
  "userFollows",
  {
    id: bigint("id", {
      mode: "bigint",
    })
      .notNull()
      .autoincrement()
      .primaryKey(),
    followerId: char("followerId", {
      length: 32,
    })
      .notNull()
      .references(() => users.id),
    followeeId: char("followeeId", {
      length: 32,
    })
      .notNull()
      .references(() => users.id),
    followAt: datetime("followAt", {
      fsp: 6,
    }).notNull(),
  },
  (t) => ({
    followUnique: unique().on(t.followerId, t.followeeId),
  })
);

export const postFavorites = mysqlTable("postFavorites", {
  id: serial("id").primaryKey(),
  userId: char("userId", {
    length: 32,
  })
    .notNull()
    .references(() => users.id),
  postId: char("postId", {
    length: 32,
  })
    .notNull()
    .references(() => posts.id),
  favoritedAt: datetime("favoritedAt", {
    fsp: 6,
  }).notNull(),
});

export const postBookmarks = mysqlTable("postBookmarks", {
  id: serial("id").primaryKey(),
  userId: char("userId", {
    length: 32,
  })
    .notNull()
    .references(() => users.id),
  postId: char("postId", {
    length: 32,
  })
    .notNull()
    .references(() => posts.id),
  bookmarkedAt: datetime("bookmarkedAt", {
    fsp: 6,
  }).notNull(),
});

export const postReactions = mysqlTable("postReactions", {
  id: serial("id").primaryKey(),
  userId: char("userId", {
    length: 32,
  })
    .notNull()
    .references(() => users.id),
  postId: char("postId", {
    length: 32,
  })
    .notNull()
    .references(() => posts.id),
  reaction: varchar("reaction", {
    length: 64,
  }).notNull(),
  reactedAt: datetime("reactedAt", {
    fsp: 6,
  }).notNull(),
});
