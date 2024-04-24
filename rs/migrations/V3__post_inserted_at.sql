DROP TABLE `posts`;

CREATE TABLE `posts` (
    `id` TEXT NOT NULL PRIMARY KEY,
    `poster_id` TEXT DEFAULT NULL,
    `content` TEXT DEFAULT NULL,
    `inserted_at` TEXT NOT NULL DEFAULT (DATETIME('now')),
    `created_at` TEXT NOT NULL DEFAULT (DATETIME('now')),
    `deleted_at` TEXT DEFAULT NULL,
    `privacy` TEXT CHECK (
        `privacy` IN ('public', 'unlisted', 'follower', 'private')
    ) NOT NULL,
    `reply_to_id` TEXT DEFAULT NULL,
    `repost_of_id` TEXT DEFAULT NULL,
    `uri` TEXT DEFAULT NULL,
    FOREIGN KEY(`poster_id`) REFERENCES `users`(`id`) ON DELETE CASCADE,
    FOREIGN KEY(`reply_to_id`) REFERENCES `posts`(`id`) ON DELETE CASCADE,
    FOREIGN KEY(`repost_of_id`) REFERENCES `posts`(`id`) ON DELETE CASCADE
);