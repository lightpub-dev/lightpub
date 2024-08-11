CREATE TABLE `users` (
    `id` CHAR(32) NOT NULL PRIMARY KEY,
    `username` VARCHAR(64) NOT NULL,
    `host` VARCHAR(128) DEFAULT NULL,
    `bpasswd` VARCHAR(60) NULL,
    `nickname` VARCHAR(255) NOT NULL,
    `avatar_id` CHAR(32) DEFAULT NULL,
    `bio` TEXT NOT NULL DEFAULT '',
    `uri` VARCHAR(512) DEFAULT NULL,
    `inbox` VARCHAR(512) DEFAULT NULL,
    `shared_inbox` VARCHAR(512) DEFAULT NULL,
    `outbox` VARCHAR(512) DEFAULT NULL,
    `private_key` TEXT DEFAULT NULL,
    `public_key` TEXT DEFAULT NULL,
    `created_at` DATETIME(6) NOT NULL DEFAULT CURRENT_TIMESTAMP(6)
);

CREATE TABLE `posts` (
    `id` CHAR(32) NOT NULL PRIMARY KEY,
    `poster_id` CHAR(32) DEFAULT NULL,
    `content` LONGTEXT DEFAULT NULL,
    `inserted_at` DATETIME(6) NOT NULL DEFAULT CURRENT_TIMESTAMP(6),
    `created_at` DATETIME(6) NOT NULL DEFAULT CURRENT_TIMESTAMP(6),
    `deleted_at` DATETIME(6) DEFAULT NULL,
    `privacy` ENUM('public', 'unlisted', 'follower', 'private') NOT NULL,
    `reply_to_id` CHAR(32) DEFAULT NULL,
    `repost_of_id` CHAR(32) DEFAULT NULL,
    `uri` VARCHAR(512) DEFAULT NULL,
    FOREIGN KEY(`poster_id`) REFERENCES `users`(`id`) ON DELETE CASCADE,
    FOREIGN KEY(`reply_to_id`) REFERENCES `posts`(`id`) ON DELETE CASCADE,
    FOREIGN KEY(`repost_of_id`) REFERENCES `posts`(`id`) ON DELETE CASCADE
);


CREATE TABLE `uploaded_files` (
    `id` CHAR(32) NOT NULL PRIMARY KEY,
    `file_ext` VARCHAR(128) NOT NULL,
    `created_at` DATETIME(6) NOT NULL DEFAULT CURRENT_TIMESTAMP(6),
    `uploaded_by_id` CHAR(32) NOT NULL,
    FOREIGN KEY(`uploaded_by_id`) REFERENCES `users`(`id`) ON DELETE CASCADE
);

CREATE TABLE `user_follows` (
    `id` BIGINT(20) NOT NULL PRIMARY KEY AUTO_INCREMENT,
    `follower_id` CHAR(32) NOT NULL,
    `followee_id` CHAR(32) NOT NULL,
    `created_at` DATETIME(6) NOT NULL DEFAULT CURRENT_TIMESTAMP(6),
    UNIQUE (`follower_id`, `followee_id`),
    FOREIGN KEY (`follower_id`) REFERENCES `users` (`id`) ON DELETE CASCADE,
    FOREIGN KEY (`followee_id`) REFERENCES `users` (`id`) ON DELETE CASCADE
);

CREATE TABLE `user_labels` (
    `user_id` CHAR(32) NOT NULL,
    `order` INT NOT NULL,
    `key` VARCHAR(64) NOT NULL,
    `value` VARCHAR(255) NOT NULL,
    PRIMARY KEY (`user_id`, `order`),
    FOREIGN KEY (`user_id`) REFERENCES `users` (`id`) ON DELETE CASCADE
);

CREATE TABLE `user_tokens` (
    `id` BIGINT(20) NOT NULL PRIMARY KEY AUTO_INCREMENT,
    `user_id` CHAR(32) DEFAULT NULL,
    `token` VARCHAR(255) NOT NULL,
    `created_at` DATETIME(6) NOT NULL DEFAULT CURRENT_TIMESTAMP(6),
    `last_used_at` DATETIME(6) NOT NULL DEFAULT CURRENT_TIMESTAMP(6),
    UNIQUE (`token`),
    FOREIGN KEY (`user_id`) REFERENCES `users` (`id`) ON DELETE CASCADE
);

CREATE TABLE `post_attachments` (
    `id` CHAR(32) NOT NULL PRIMARY KEY,
    `post_id` CHAR(32) DEFAULT NULL,
    `uploaded_file_id` CHAR(32) NOT NULL,
    FOREIGN KEY (`post_id`) REFERENCES `posts` (`id`) ON DELETE CASCADE,
    FOREIGN KEY (`uploaded_file_id`) REFERENCES `uploaded_files` (`id`) ON DELETE CASCADE
);

CREATE TABLE `post_favorites` (
    `id` CHAR(32) NOT NULL PRIMARY KEY,
    `post_id` CHAR(32) NOT NULL,
    `user_id` CHAR(32) NOT NULL,
    `is_bookmark` BOOLEAN NOT NULL,
    `created_at` DATETIME(6) NOT NULL DEFAULT CURRENT_TIMESTAMP(6),
    UNIQUE (`post_id`, `user_id`, `is_bookmark`),
    FOREIGN KEY (`post_id`) REFERENCES `posts` (`id`),
    FOREIGN KEY (`user_id`) REFERENCES `users` (`id`)
);

CREATE TABLE `post_hashtags` (
    `id` BIGINT(20) NOT NULL PRIMARY KEY AUTO_INCREMENT,
    `post_id` CHAR(32) NOT NULL,
    `hashtag_name` VARCHAR(255) NOT NULL,
    FOREIGN KEY(`post_id`) REFERENCES `posts`(`id`) ON DELETE CASCADE
);

CREATE TABLE `post_mentions` (
    `id` BIGINT(20) NOT NULL PRIMARY KEY AUTO_INCREMENT,
    `post_id` CHAR(32) NOT NULL,
    `target_user_id` CHAR(32) NOT NULL,
    FOREIGN KEY(`post_id`) REFERENCES `posts`(`id`) ON DELETE CASCADE,
    FOREIGN KEY(`target_user_id`) REFERENCES `users`(`id`) ON DELETE CASCADE
);

CREATE TABLE `reactions` (
    `id` BIGINT(20) NOT NULL PRIMARY KEY AUTO_INCREMENT,
    `name` VARCHAR(64) NOT NULL
);

CREATE TABLE `post_reactions` (
    `id` CHAR(32) NOT NULL PRIMARY KEY,
    `post_id` CHAR(32) NOT NULL,
    `user_id` CHAR(32) NOT NULL,
    `custom_reaction_id` BIGINT(20) NULL DEFAULT NULL,
    `reaction_str` VARCHAR(255) COLLATE utf8mb4_bin NULL DEFAULT NULL,
    `created_at` DATETIME(6) NOT NULL DEFAULT CURRENT_TIMESTAMP(6),
    FOREIGN KEY (`post_id`) REFERENCES `posts` (`id`) ON DELETE CASCADE,
    FOREIGN KEY (`custom_reaction_id`) REFERENCES `reactions` (`id`),
    FOREIGN KEY (`user_id`) REFERENCES `users` (`id`)
);

CREATE TABLE `user_follow_requests` (
    `id` CHAR(32) NOT NULL PRIMARY KEY,
    `uri` VARCHAR(512) NULL UNIQUE,
    `incoming` BOOLEAN NOT NULL,
    `follower_id` CHAR(32) NOT NULL,
    `followee_id` CHAR(32) NOT NULL,
    `created_at` DATETIME(6) NOT NULL DEFAULT CURRENT_TIMESTAMP(6),
    UNIQUE (`follower_id`, `followee_id`),
    FOREIGN KEY(`follower_id`) REFERENCES `users`(`id`) ON DELETE CASCADE,
    FOREIGN KEY(`followee_id`) REFERENCES `users`(`id`) ON DELETE CASCADE
);

CREATE TABLE `remote_user_details` (
    `id` CHAR(32) NOT NULL PRIMARY KEY,
    `following_uri` VARCHAR(512) DEFAULT NULL,
    `followers_uri` VARCHAR(512) DEFAULT NULL,
    `liked_uri` VARCHAR(512) DEFAULT NULL,
    `fetched_at` DATETIME(6) NOT NULL DEFAULT CURRENT_TIMESTAMP(6),
    FOREIGN KEY (`id`) REFERENCES `users` (`id`) ON DELETE CASCADE
);

CREATE TABLE `user_keys` (
    `id` VARCHAR(512) NOT NULL PRIMARY KEY,
    `owner_id` CHAR(32) NOT NULL,
    `public_key` TEXT NOT NULL,
    `updated_at` DATETIME(6) NOT NULL DEFAULT CURRENT_TIMESTAMP(6),
    FOREIGN KEY (`owner_id`) REFERENCES `users` (`id`) ON DELETE CASCADE
);

CREATE TABLE `remote_users` (
    `user_id` CHAR(32) NOT NULL PRIMARY KEY,
    `following` VARCHAR(512) NULL DEFAULT NULL,
    `followers` VARCHAR(512) NULL DEFAULT NULL,
    `liked` VARCHAR(512) NULL DEFAULT NULL,
    `fetched_at` DATETIME(6) NOT NULL DEFAULT CURRENT_TIMESTAMP(6),
    FOREIGN KEY(`user_id`) REFERENCES `users`(`id`) ON DELETE CASCADE
);

CREATE TABLE `QueuedTask` (
    `id` BIGINT(20) NOT NULL PRIMARY KEY AUTO_INCREMENT,
    `created_at` DATETIME(6) NOT NULL DEFAULT CURRENT_TIMESTAMP(6),
    `started_at` DATETIME(6) DEFAULT NULL,
    `current_retry` INT NOT NULL DEFAULT 0,
    `max_retry` INT NOT NULL,
    `payload` LONGTEXT NOT NULL
);
