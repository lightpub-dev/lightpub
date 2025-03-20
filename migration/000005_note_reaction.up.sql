-- Create note_reactions table
CREATE TABLE `note_reactions` (
    `id` INTEGER NOT NULL AUTO_INCREMENT,
    `note_id` binary(16) NOT NULL,
    `user_id` binary(16) NOT NULL,
    `reaction` VARCHAR(64) COLLATE utf8mb4_bin NOT NULL,
    `created_at` datetime(6) NOT NULL DEFAULT current_timestamp(6),
    PRIMARY KEY (`id`),
    UNIQUE KEY `idx_note_reactions_unique` (`note_id`, `user_id`),
    KEY `fk_note_reactions_user_id` (`user_id`),
    CONSTRAINT `fk_note_reactions_note_id` FOREIGN KEY (`note_id`) REFERENCES `notes` (`id`) ON DELETE CASCADE,
    CONSTRAINT `fk_note_reactions_user_id` FOREIGN KEY (`user_id`) REFERENCES `users` (`id`) ON DELETE CASCADE
) ENGINE = InnoDB DEFAULT CHARSET = utf8mb4 COLLATE = utf8mb4_general_ci;

-- Create note_bookmarks table
CREATE TABLE `note_bookmarks` (
    `id` INTEGER NOT NULL AUTO_INCREMENT,
    `note_id` binary(16) NOT NULL,
    `user_id` binary(16) NOT NULL,
    `created_at` datetime(6) NOT NULL DEFAULT current_timestamp(6),
    PRIMARY KEY (`id`),
    UNIQUE KEY `idx_note_bookmarks_unique` (`note_id`, `user_id`),
    KEY `fk_note_bookmarks_user_id` (`user_id`),
    CONSTRAINT `fk_note_bookmarks_note_id` FOREIGN KEY (`note_id`) REFERENCES `notes` (`id`) ON DELETE CASCADE,
    CONSTRAINT `fk_note_bookmarks_user_id` FOREIGN KEY (`user_id`) REFERENCES `users` (`id`) ON DELETE CASCADE
) ENGINE = InnoDB DEFAULT CHARSET = utf8mb4 COLLATE = utf8mb4_general_ci;

-- Copy data from note_likes to note_reactions (is_private=0)
INSERT INTO
    `note_reactions` (`note_id`, `user_id`, `reaction`, `created_at`)
SELECT
    `note_id`,
    `user_id`,
    '♥️',
    `created_at`
FROM
    `note_likes`
WHERE
    `is_private` = 0;

-- Copy data from note_likes to note_bookmarks (is_private=1)
INSERT INTO
    `note_bookmarks` (`note_id`, `user_id`, `created_at`)
SELECT
    `note_id`,
    `user_id`,
    `created_at`
FROM
    `note_likes`
WHERE
    `is_private` = 1;

-- Drop the old note_likes table
DROP TABLE `note_likes`;