CREATE TABLE `users` (
    `id` TEXT NOT NULL PRIMARY KEY,
    `username` TEXT NOT NULL,
    `host` TEXT DEFAULT NULL,
    `bpasswd` TEXT NULL,
    `nickname` TEXT NOT NULL,
    `avatar_id` TEXT DEFAULT NULL,
    `bio` TEXT NOT NULL DEFAULT '',
    `uri` TEXT DEFAULT NULL,
    `inbox` TEXT DEFAULT NULL,
    `shared_inbox` TEXT DEFAULT NULL,
    `outbox` TEXT DEFAULT NULL,
    `private_key` TEXT DEFAULT NULL,
    `public_key` TEXT DEFAULT NULL,
    `created_at` TEXT NOT NULL DEFAULT (DATETIME('now'))
);

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


CREATE TABLE `uploaded_files` (
    `id` TEXT NOT NULL PRIMARY KEY,
    `file_ext` TEXT NOT NULL,
    `created_at` TEXT NOT NULL DEFAULT (DATETIME('now')),
    `uploaded_by_id` TEXT NOT NULL,
    FOREIGN KEY(`uploaded_by_id`) REFERENCES `users`(`id`) ON DELETE CASCADE
);

CREATE TABLE `user_follows` (
    `id` INTEGER NOT NULL PRIMARY KEY,
    `follower_id` TEXT NOT NULL,
    `followee_id` TEXT NOT NULL,
    `created_at` TEXT NOT NULL DEFAULT (DATETIME('now')),
    UNIQUE (`follower_id`, `followee_id`),
    FOREIGN KEY (`follower_id`) REFERENCES `users` (`id`) ON DELETE CASCADE,
    FOREIGN KEY (`followee_id`) REFERENCES `users` (`id`) ON DELETE CASCADE
);

CREATE TABLE `user_labels` (
    `user_id` TEXT NOT NULL,
    `order` INTEGER NOT NULL,
    `key` text NOT NULL,
    `value` text NOT NULL,
    PRIMARY KEY (`user_id`, `order`),
    FOREIGN KEY (`user_id`) REFERENCES `users` (`id`) ON DELETE CASCADE
);

CREATE TABLE `user_tokens` (
    `id` INTEGER NOT NULL PRIMARY KEY,
    `user_id` TEXT DEFAULT NULL,
    `token` TEXT NOT NULL,
    `created_at` TEXT NOT NULL DEFAULT (DATETIME('now')),
    `last_used_at` TEXT NOT NULL DEFAULT (DATETIME('now')),
    UNIQUE (`token`),
    FOREIGN KEY (`user_id`) REFERENCES `users` (`id`) ON DELETE CASCADE
);

CREATE TABLE `post_attachments` (
    `id` TEXT NOT NULL PRIMARY KEY,
    `post_id` TEXT DEFAULT NULL,
    `uploaded_file_id` TEXT NOT NULL,
    FOREIGN KEY (`post_id`) REFERENCES `posts` (`id`) ON DELETE CASCADE,
    FOREIGN KEY (`uploaded_file_id`) REFERENCES `uploaded_files` (`id`) ON DELETE CASCADE
);

CREATE TABLE `post_favorites` (
    `id` TEXT NOT NULL PRIMARY KEY,
    `post_id` TEXT NOT NULL,
    `user_id` TEXT NOT NULL,
    `is_bookmark` BOOLEAN NOT NULL,
    `created_at` TEXT NOT NULL DEFAULT (DATETIME('now')),
    UNIQUE (`post_id`, `user_id`, `is_bookmark`),
    FOREIGN KEY (`post_id`) REFERENCES `posts` (`id`),
    FOREIGN KEY (`user_id`) REFERENCES `users` (`id`)
);

CREATE TABLE `post_hashtags` (
    `id` INTEGER NOT NULL PRIMARY KEY,
    `post_id` TEXT NOT NULL,
    `hashtag_name` TEXT NOT NULL,
    FOREIGN KEY(`post_id`) REFERENCES `posts`(`id`) ON DELETE CASCADE
);

CREATE TABLE `post_mentions` (
    `id` INTEGER NOT NULL PRIMARY KEY,
    `post_id` TEXT NOT NULL,
    `target_user_id` TEXT NOT NULL,
    FOREIGN KEY(`post_id`) REFERENCES `posts`(`id`) ON DELETE CASCADE,
    FOREIGN KEY(`target_user_id`) REFERENCES `users`(`id`) ON DELETE CASCADE
);

create table `reactions` (
    `id` INTEGER not null PRIMARY KEY,
    `name` TEXT not null
);

CREATE TABLE `post_reactions` (
    `id` TEXT NOT NULL PRIMARY KEY,
    `post_id` TEXT NOT NULL,
    `user_id` TEXT NOT NULL,
    `custom_reaction_id` INTEGER NULL DEFAULT NULL,
    `reaction_str` TEXT NULL DEFAULT NULL,
    `created_at` TEXT NOT NULL DEFAULT (DATETIME('now')),
    FOREIGN KEY (`post_id`) REFERENCES `posts` (`id`) ON DELETE CASCADE,
    FOREIGN KEY (`custom_reaction_id`) REFERENCES `reactions` (`id`),
    FOREIGN KEY (`user_id`) REFERENCES `users` (`id`)
);

CREATE TABLE user_follow_requests(
    id TEXT NOT NULL PRIMARY KEY,
    uri TEXT NULL UNIQUE,
    incoming BOOLEAN NOT NULL,
    follower_id TEXT NOT NULL,
    followee_id TEXT NOT NULL,
    created_at TEXT NOT NULL DEFAULT (DATETIME('now')),
    UNIQUE (`follower_id`, `followee_id`),
    FOREIGN KEY(`follower_id`) REFERENCES `users`(`id`) ON DELETE CASCADE,
    FOREIGN KEY(`followee_id`) REFERENCES `users`(`id`) ON DELETE CASCADE
);

CREATE TABLE `remote_user_details` (
    `id` TEXT NOT NULL PRIMARY KEY,
    `following_uri` TEXT DEFAULT NULL,
    `followers_uri` TEXT DEFAULT NULL,
    `liked_uri` TEXT DEFAULT NULL,
    `fetched_at` TEXT NOT NULL DEFAULT (DATETIME('now')),
    FOREIGN KEY (`id`) REFERENCES `users` (`id`) ON DELETE CASCADE
);

CREATE TABLE `user_keys` (
    id TEXT NOT NULL PRIMARY KEY,
    owner_id TEXT NOT NULL,
    public_key TEXT NOT NULL,
    updated_at TEXT NOT NULL DEFAULT (DATETIME('now')),
    FOREIGN KEY (`owner_id`) REFERENCES `users` (`id`) ON DELETE CASCADE
);

CREATE TABLE remote_users(
    user_id TEXT NOT NULL PRIMARY KEY,
    following TEXT NULL DEFAULT NULL,
    followers TEXT NULL DEFAULT NULL,
    liked TEXT NULL DEFAULT NULL,
    fetched_at TEXT NOT NULL DEFAULT (DATETIME('now')),
    FOREIGN KEY(`user_id`) REFERENCES `users`(`id`) ON DELETE CASCADE
);
