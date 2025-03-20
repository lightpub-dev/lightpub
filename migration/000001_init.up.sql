-- 1. Tables with no foreign key dependencies
CREATE TABLE `apub_error_report` (
    `id` INTEGER NOT NULL AUTO_INCREMENT,
    `activity` text NOT NULL,
    `error_msg` text NOT NULL,
    `received_at` datetime(6) NOT NULL DEFAULT current_timestamp(6),
    PRIMARY KEY (`id`)
) ENGINE = InnoDB DEFAULT CHARSET = utf8mb4 COLLATE = utf8mb4_general_ci;

CREATE TABLE `tag` (
    `id` INTEGER NOT NULL AUTO_INCREMENT,
    `name` varchar(256) NOT NULL,
    PRIMARY KEY (`id`)
) ENGINE = InnoDB DEFAULT CHARSET = utf8mb4 COLLATE = utf8mb4_general_ci;

CREATE TABLE `upload` (
    `id` binary(16) NOT NULL,
    `filename` varchar(64) DEFAULT NULL,
    `url` varchar(512) DEFAULT NULL,
    `mime_type` varchar(255) NOT NULL,
    PRIMARY KEY (`id`)
) ENGINE = InnoDB DEFAULT CHARSET = utf8mb4 COLLATE = utf8mb4_general_ci;

-- 2. Create user table (depends on upload)
CREATE TABLE `user` (
    `id` binary(16) NOT NULL,
    `username` varchar(128) NOT NULL,
    `domain` varchar(128) NOT NULL,
    `password` varchar(128) DEFAULT NULL,
    `nickname` varchar(255) NOT NULL,
    `bio` text NOT NULL,
    `avatar` binary(16) DEFAULT NULL,
    `url` varchar(512) DEFAULT NULL,
    `inbox` varchar(512) DEFAULT NULL,
    `shared_inbox` varchar(512) DEFAULT NULL,
    `outbox` varchar(512) DEFAULT NULL,
    `private_key` text DEFAULT NULL,
    `public_key` text DEFAULT NULL,
    `created_at` datetime(6) DEFAULT NULL,
    `fetched_at` datetime(6) DEFAULT NULL,
    `view_url` varchar(512) DEFAULT NULL,
    `following` varchar(512) DEFAULT NULL,
    `followers` varchar(512) DEFAULT NULL,
    `auto_follow_accept` tinyint(1) NOT NULL DEFAULT 1,
    `auth_expired_at` datetime(6) DEFAULT NULL,
    `is_bot` tinyint(1) NOT NULL DEFAULT 0,
    `is_admin` tinyint(1) NOT NULL DEFAULT 0,
    `hide_follows` tinyint(1) NOT NULL DEFAULT 0,
    `preferred_inbox` varchar(512) GENERATED ALWAYS AS (coalesce(`shared_inbox`, `inbox`)) STORED,
    PRIMARY KEY (`id`),
    UNIQUE KEY `user_unique_username` (`username`, `domain`),
    UNIQUE KEY `idx_user_url_unique` (`url`),
    KEY `fk_user_avatar` (`avatar`),
    CONSTRAINT `fk_user_avatar` FOREIGN KEY (`avatar`) REFERENCES `upload` (`id`) ON DELETE
    SET
        NULL ON UPDATE CASCADE
) ENGINE = InnoDB DEFAULT CHARSET = utf8mb4 COLLATE = utf8mb4_general_ci;

-- 3. Create note table (depends on user)
CREATE TABLE `note` (
    `id` binary(16) NOT NULL,
    `url` varchar(512) DEFAULT NULL,
    `view_url` varchar(512) DEFAULT NULL,
    `author_id` binary(16) NOT NULL,
    `content` text DEFAULT NULL,
    `content_type` varchar(32) DEFAULT NULL,
    `created_at` datetime(6) NOT NULL,
    `inserted_at` datetime(6) NOT NULL DEFAULT current_timestamp(6),
    `updated_at` datetime(6) DEFAULT NULL,
    `deleted_at` datetime(6) DEFAULT NULL,
    `visibility` enum('public', 'unlisted', 'follower', 'private') NOT NULL,
    `reply_to_id` binary(16) DEFAULT NULL,
    `renote_of_id` binary(16) DEFAULT NULL,
    `sensitive` tinyint(1) NOT NULL DEFAULT 0,
    `fetched_at` datetime(6) DEFAULT NULL,
    PRIMARY KEY (`id`),
    KEY `fk_note_author_id` (`author_id`),
    KEY `idx_note_created_at` (`created_at` DESC),
    KEY `idx_note_reply_to_id` (`reply_to_id`),
    KEY `idx_note_renote_of_id` (`renote_of_id`),
    CONSTRAINT `fk_note_author_id` FOREIGN KEY (`author_id`) REFERENCES `user` (`id`) ON DELETE CASCADE ON UPDATE CASCADE
) ENGINE = InnoDB DEFAULT CHARSET = utf8mb4 COLLATE = utf8mb4_general_ci;

-- 4. Tables that depend on note and/or user
CREATE TABLE `note_like` (
    `id` INTEGER NOT NULL AUTO_INCREMENT,
    `note_id` binary(16) NOT NULL,
    `user_id` binary(16) NOT NULL,
    `is_private` tinyint(1) NOT NULL,
    `created_at` datetime(6) NOT NULL DEFAULT current_timestamp(6),
    PRIMARY KEY (`id`),
    UNIQUE KEY `idx_note_like_unique` (`note_id`, `user_id`, `is_private`),
    KEY `fk_note_like_user_id` (`user_id`),
    CONSTRAINT `fk_note_like_note_id` FOREIGN KEY (`note_id`) REFERENCES `note` (`id`) ON DELETE CASCADE,
    CONSTRAINT `fk_note_like_user_id` FOREIGN KEY (`user_id`) REFERENCES `user` (`id`) ON DELETE CASCADE
) ENGINE = InnoDB DEFAULT CHARSET = utf8mb4 COLLATE = utf8mb4_general_ci;

CREATE TABLE `note_mention` (
    `id` INTEGER NOT NULL AUTO_INCREMENT,
    `note_id` binary(16) NOT NULL,
    `target_user_id` binary(16) NOT NULL,
    PRIMARY KEY (`id`),
    UNIQUE KEY `idx_note_mention_unique` (`note_id`, `target_user_id`),
    KEY `fk_note_mention_user_id` (`target_user_id`),
    CONSTRAINT `fk_note_mention_note_id` FOREIGN KEY (`note_id`) REFERENCES `note` (`id`) ON DELETE CASCADE ON UPDATE CASCADE,
    CONSTRAINT `fk_note_mention_user_id` FOREIGN KEY (`target_user_id`) REFERENCES `user` (`id`) ON DELETE CASCADE ON UPDATE CASCADE
) ENGINE = InnoDB AUTO_INCREMENT = 4 DEFAULT CHARSET = utf8mb4 COLLATE = utf8mb4_general_ci;

CREATE TABLE `note_tag` (
    `id` INTEGER NOT NULL AUTO_INCREMENT,
    `note_id` binary(16) NOT NULL,
    `tag_id` INTEGER NOT NULL,
    PRIMARY KEY (`id`),
    UNIQUE KEY `idx_note_tag_unique` (`note_id`, `tag_id`),
    KEY `fk_note_tag_tag_id` (`tag_id`),
    CONSTRAINT `fk_note_tag_note_id` FOREIGN KEY (`note_id`) REFERENCES `note` (`id`) ON DELETE CASCADE ON UPDATE CASCADE,
    CONSTRAINT `fk_note_tag_tag_id` FOREIGN KEY (`tag_id`) REFERENCES `tag` (`id`) ON DELETE CASCADE ON UPDATE CASCADE
) ENGINE = InnoDB DEFAULT CHARSET = utf8mb4 COLLATE = utf8mb4_general_ci;

CREATE TABLE `note_upload` (
    `id` INTEGER NOT NULL AUTO_INCREMENT,
    `note_id` binary(16) NOT NULL,
    `upload_id` binary(16) NOT NULL,
    PRIMARY KEY (`id`),
    KEY `fk_note_upload_note_id` (`note_id`),
    KEY `fk_note_upload_upload_id` (`upload_id`),
    CONSTRAINT `fk_note_upload_note_id` FOREIGN KEY (`note_id`) REFERENCES `note` (`id`) ON DELETE CASCADE ON UPDATE CASCADE,
    CONSTRAINT `fk_note_upload_upload_id` FOREIGN KEY (`upload_id`) REFERENCES `upload` (`id`) ON DELETE CASCADE ON UPDATE CASCADE
) ENGINE = InnoDB DEFAULT CHARSET = utf8mb4 COLLATE = utf8mb4_general_ci;

CREATE TABLE `notification` (
    `id` INTEGER NOT NULL AUTO_INCREMENT,
    `user_id` binary(16) NOT NULL,
    `body` longtext CHARACTER SET utf8mb4 COLLATE utf8mb4_bin NOT NULL CHECK (json_valid(`body`)),
    `created_at` datetime(6) NOT NULL DEFAULT current_timestamp(6),
    `read_at` datetime(6) DEFAULT NULL,
    PRIMARY KEY (`id`),
    KEY `idx_notification_read_at` (`user_id`, `read_at`),
    CONSTRAINT `fk_notification_user_id` FOREIGN KEY (`user_id`) REFERENCES `user` (`id`) ON DELETE CASCADE
) ENGINE = InnoDB AUTO_INCREMENT = 12 DEFAULT CHARSET = utf8mb4 COLLATE = utf8mb4_general_ci;

CREATE TABLE `push_notification` (
    `id` INTEGER NOT NULL AUTO_INCREMENT,
    `user_id` binary(16) NOT NULL,
    `endpoint` varchar(512) NOT NULL,
    `p256dh` text NOT NULL,
    `auth` text NOT NULL,
    `created_at` datetime(6) NOT NULL DEFAULT current_timestamp(6),
    PRIMARY KEY (`id`),
    UNIQUE KEY `idx_push_notification_unique` (`user_id`, `endpoint`),
    CONSTRAINT `fk_push_notification_user_id` FOREIGN KEY (`user_id`) REFERENCES `user` (`id`) ON DELETE CASCADE
) ENGINE = InnoDB AUTO_INCREMENT = 9 DEFAULT CHARSET = utf8mb4 COLLATE = utf8mb4_general_ci;

CREATE TABLE `remote_public_key` (
    `id` INTEGER NOT NULL AUTO_INCREMENT,
    `owner_id` binary(16) NOT NULL,
    `key_id` varchar(512) NOT NULL,
    `public_key` text NOT NULL,
    PRIMARY KEY (`id`),
    KEY `fk_remote_public_key_owner_id` (`owner_id`),
    KEY `idx_remote_public_key_key_id_unique` (`key_id`),
    CONSTRAINT `fk_remote_public_key_owner_id` FOREIGN KEY (`owner_id`) REFERENCES `user` (`id`) ON DELETE CASCADE ON UPDATE CASCADE
) ENGINE = InnoDB AUTO_INCREMENT = 2 DEFAULT CHARSET = utf8mb4 COLLATE = utf8mb4_general_ci;

CREATE TABLE `user_block` (
    `id` INTEGER NOT NULL AUTO_INCREMENT,
    `blocker_id` binary(16) NOT NULL,
    `blocked_id` binary(16) NOT NULL,
    `blocked_at` datetime(6) NOT NULL DEFAULT current_timestamp(6),
    PRIMARY KEY (`id`),
    KEY `fk_user_block_blocker_id` (`blocker_id`),
    KEY `fk_user_block_blocked_id` (`blocked_id`),
    CONSTRAINT `fk_user_block_blocked_id` FOREIGN KEY (`blocked_id`) REFERENCES `user` (`id`) ON DELETE CASCADE ON UPDATE CASCADE,
    CONSTRAINT `fk_user_block_blocker_id` FOREIGN KEY (`blocker_id`) REFERENCES `user` (`id`) ON DELETE CASCADE ON UPDATE CASCADE
) ENGINE = InnoDB DEFAULT CHARSET = utf8mb4 COLLATE = utf8mb4_general_ci;

CREATE TABLE `user_follow` (
    `id` INTEGER NOT NULL AUTO_INCREMENT,
    `follower_id` binary(16) NOT NULL,
    `followed_id` binary(16) NOT NULL,
    `pending` tinyint(1) NOT NULL,
    `url` varchar(512) DEFAULT NULL,
    `created_at` datetime(6) NOT NULL DEFAULT current_timestamp(6),
    PRIMARY KEY (`id`),
    UNIQUE KEY `user_follow_unique` (`follower_id`, `followed_id`),
    KEY `fk_user_follow_followed_id` (`followed_id`),
    CONSTRAINT `fk_user_follow_followed_id` FOREIGN KEY (`followed_id`) REFERENCES `user` (`id`) ON DELETE CASCADE ON UPDATE CASCADE,
    CONSTRAINT `fk_user_follow_follower_id` FOREIGN KEY (`follower_id`) REFERENCES `user` (`id`) ON DELETE CASCADE ON UPDATE CASCADE
) ENGINE = InnoDB AUTO_INCREMENT = 3 DEFAULT CHARSET = utf8mb4 COLLATE = utf8mb4_general_ci;