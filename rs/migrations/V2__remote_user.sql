CREATE TABLE remote_users(
    user_id TEXT NOT NULL PRIMARY KEY,
    following TEXT NULL DEFAULT NULL,
    followers TEXT NULL DEFAULT NULL,
    liked TEXT NULL DEFAULT NULL,
    fetched_at TEXT NOT NULL DEFAULT (DATETIME('now')),
    FOREIGN KEY(`user_id`) REFERENCES `users`(`id`) ON DELETE CASCADE
);
