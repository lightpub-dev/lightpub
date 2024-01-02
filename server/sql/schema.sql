-- Schema definitions

CREATE TABLE IF NOT EXISTS `User` (
  `id` varbinary(16) NOT NULL,
  `username` varchar(64) NOT NULL,
  `host` varchar(128) NOT NULL DEFAULT '',
  `bpassword` varchar(60) DEFAULT NULL,
  `nickname` varchar(255) NOT NULL,
  `url` varchar(512) DEFAULT NULL,
  `inbox` varchar(512) DEFAULT NULL,
  `outbox` varchar(512) DEFAULT NULL,
  `is_local` tinyint GENERATED ALWAYS AS ((`host` = _utf8mb4'')) VIRTUAL,
  PRIMARY KEY (`id`),
  UNIQUE KEY `username_UNIQUE` (`username`)
) ENGINE=InnoDB DEFAULT CHARSET=utf8mb4 COLLATE=utf8mb4_0900_ai_ci;

CREATE TABLE IF NOT EXISTS `UserFollow` (
  `id` bigint NOT NULL AUTO_INCREMENT,
  `follower_id` varbinary(16) NOT NULL,
  `followee_id` varbinary(16) NOT NULL,
  `created_at` datetime DEFAULT CURRENT_TIMESTAMP,
  PRIMARY KEY (`id`)
) ENGINE=InnoDB DEFAULT CHARSET=utf8mb4 COLLATE=utf8mb4_0900_ai_ci;

CREATE TABLE IF NOT EXISTS `UserToken` (
  `id` bigint NOT NULL AUTO_INCREMENT,
  `user_id` varbinary(16) NOT NULL,
  `token` varchar(64) NOT NULL,
  PRIMARY KEY (`id`)
) ENGINE=InnoDB DEFAULT CHARSET=utf8mb4 COLLATE=utf8mb4_0900_ai_ci;

CREATE TABLE IF NOT EXISTS `Post` (
  `id` varbinary(16) NOT NULL,
  `poster_id` varbinary(16) NOT NULL,
  `content` longtext NOT NULL,
  `inserted_at` datetime NOT NULL,
  `created_at` datetime NOT NULL,
  `privacy` enum('public','unlisted','follower','private') NOT NULL,
  `reply_to` varbinary(16) DEFAULT NULL,
  `repost_of` varbinary(16) DEFAULT NULL,
  `quote_of` varbinary(16) DEFAULT NULL,
  `poll_id` varbinary(16) DEFAULT NULL,
  `scheduled_at` datetime DEFAULT NULL,
  `edited` tinyint NOT NULL DEFAULT '0',
  PRIMARY KEY (`id`)
) ENGINE=InnoDB DEFAULT CHARSET=utf8mb4 COLLATE=utf8mb4_0900_ai_ci;

CREATE TABLE IF NOT EXISTS `PostHashtag` (
  `id` bigint NOT NULL,
  `post_id` varbinary(16) NOT NULL,
  `hashtag_name` varchar(255) NOT NULL,
  PRIMARY KEY (`id`)
) ENGINE=InnoDB DEFAULT CHARSET=utf8mb4 COLLATE=utf8mb4_0900_ai_ci;

CREATE TABLE IF NOT EXISTS `PostFavorite` (
  `id` bigint NOT NULL AUTO_INCREMENT,
  `post_id` varbinary(16) NOT NULL,
  `user_id` varbinary(16) NOT NULL,
  `is_bookmark` tinyint NOT NULL,
  PRIMARY KEY (`id`),
  UNIQUE KEY `post_favorite_favorite_unique` (`post_id`,`user_id`,`is_bookmark`) USING BTREE
) ENGINE=InnoDB DEFAULT CHARSET=utf8mb4 COLLATE=utf8mb4_0900_ai_ci;

CREATE TABLE IF NOT EXISTS `PostMention` (
  `id` bigint NOT NULL AUTO_INCREMENT,
  `post_id` varbinary(16) NOT NULL,
  `target_user_id` varbinary(16) NOT NULL,
  PRIMARY KEY (`id`)
) ENGINE=InnoDB DEFAULT CHARSET=utf8mb4 COLLATE=utf8mb4_0900_ai_ci;

CREATE TABLE IF NOT EXISTS `PostReaction` (
  `id` bigint NOT NULL AUTO_INCREMENT,
  `post_id` varbinary(16) NOT NULL,
  `reaction` varchar(128) NOT NULL,
  `user_id` varbinary(16) NOT NULL,
  PRIMARY KEY (`id`),
  UNIQUE KEY `post_reaction_unique` (`post_id`,`user_id`,`reaction`) USING BTREE
) ENGINE=InnoDB DEFAULT CHARSET=utf8mb4 COLLATE=utf8mb4_0900_ai_ci;

CREATE TABLE IF NOT EXISTS `PostAttachment` (
  `id` varbinary(16) NOT NULL,
  `post_id` varbinary(16) NOT NULL,
  `file_ext` varchar(128) NOT NULL,
  PRIMARY KEY (`id`)
) ENGINE=InnoDB DEFAULT CHARSET=utf8mb4 COLLATE=utf8mb4_0900_ai_ci;

CREATE TABLE IF NOT EXISTS `PostPoll` (
  `id` varbinary(16) NOT NULL,
  `allow_multiple` tinyint NOT NULL,
  `due` datetime DEFAULT NULL,
  PRIMARY KEY (`id`)
) ENGINE=InnoDB DEFAULT CHARSET=utf8mb4 COLLATE=utf8mb4_0900_ai_ci;

CREATE TABLE IF NOT EXISTS `PollChoice` (
  `id` bigint NOT NULL AUTO_INCREMENT,
  `poll_id` varbinary(16) NOT NULL,
  `title` text NOT NULL,
  `count` bigint NOT NULL DEFAULT '0',
  PRIMARY KEY (`id`)
) ENGINE=InnoDB DEFAULT CHARSET=utf8mb4 COLLATE=utf8mb4_0900_ai_ci;

CREATE TABLE IF NOT EXISTS `PollVote` (
  `id` bigint NOT NULL AUTO_INCREMENT,
  `poll_id` varbinary(16) NOT NULL,
  `user_id` varbinary(16) NOT NULL,
  PRIMARY KEY (`id`)
) ENGINE=InnoDB DEFAULT CHARSET=utf8mb4 COLLATE=utf8mb4_0900_ai_ci;

-- Index definitions
CREATE INDEX `post_mention_post_id_idx` ON PostMention (`post_id`,`target_user_id`);

CREATE INDEX `post_poster_privacy_time_idx` ON Post (`poster_id`,`privacy`,`created_at` DESC);

-- Stored function definitions
CREATE DEFINER=`root`@`%` FUNCTION `CreateUserURL`(
	username VARCHAR(64),
	url VARCHAR(512),
    localUserUrl VARCHAR(128)
) RETURNS varchar(512) CHARSET utf8mb4
    DETERMINISTIC
BEGIN
IF url IS NOT NULL
THEN
	RETURN url;
ELSE
	RETURN CONCAT(localUserUrl, '/', username);
END IF;
END
