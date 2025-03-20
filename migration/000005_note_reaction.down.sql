-- Recreate the original note_likes table
CREATE TABLE `note_likes` (
    `id` INTEGER NOT NULL AUTO_INCREMENT,
    `note_id` binary(16) NOT NULL,
    `user_id` binary(16) NOT NULL,
    `is_private` tinyint(1) NOT NULL,
    `created_at` datetime(6) NOT NULL DEFAULT current_timestamp(6),
    PRIMARY KEY (`id`),
    UNIQUE KEY `idx_note_likes_unique` (`note_id`, `user_id`, `is_private`),
    KEY `fk_note_likes_user_id` (`user_id`),
    CONSTRAINT `fk_note_likes_note_id` FOREIGN KEY (`note_id`) REFERENCES `notes` (`id`) ON DELETE CASCADE,
    CONSTRAINT `fk_note_likes_user_id` FOREIGN KEY (`user_id`) REFERENCES `users` (`id`) ON DELETE CASCADE
) ENGINE = InnoDB DEFAULT CHARSET = utf8mb4 COLLATE = utf8mb4_general_ci;

-- Copy data from note_reactions to note_likes (setting is_private=0)
INSERT INTO
    `note_likes` (`note_id`, `user_id`, `is_private`, `created_at`)
SELECT
    `note_id`,
    `user_id`,
    0,
    `created_at`
FROM
    `note_reactions`;

-- Copy data from note_bookmarks to note_likes (setting is_private=1)
INSERT INTO
    `note_likes` (`note_id`, `user_id`, `is_private`, `created_at`)
SELECT
    `note_id`,
    `user_id`,
    1,
    `created_at`
FROM
    `note_bookmarks`;

-- Drop the new tables
DROP TABLE `note_reactions`;

DROP TABLE `note_bookmarks`;