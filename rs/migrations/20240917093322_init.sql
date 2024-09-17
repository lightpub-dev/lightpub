-- Add migration script here
CREATE TABLE `users` (
    `id` CHAR(32) NOT NULL,
    `username` varchar(64) NOT NULL,
    `host` varchar(128) DEFAULT NULL,
    `bpasswd` varchar(60) NOT NULL,
    `nickname` varchar(255) NOT NULL,
    `avatar_id` CHAR(32) NOT NULL,
    `bio` text NOT NULL,
    `url` varchar(512) DEFAULT NULL,
    `inbox` varchar(512) DEFAULT NULL,
    `outbox` varchar(512) DEFAULT NULL,
    `private_key` text DEFAULT NULL,
    `public_key` text DEFAULT NULL,
    `created_at` datetime(6) NOT NULL,
    PRIMARY KEY (`id`),
    UNIQUE KEY `idx_users_username` (`username`)
);

CREATE TABLE `posts` (
    `id` CHAR(32) NOT NULL,
    `poster_id` CHAR(32) DEFAULT NULL,
    `content` longtext DEFAULT NULL,
    `inserted_at` datetime(6) NOT NULL,
    `created_at` datetime(6) NOT NULL,
    `deleted_at` datetime(6) DEFAULT NULL,
    `privacy` enum('public', 'unlisted', 'follower', 'private') NOT NULL,
    `reply_to_id` CHAR(32) DEFAULT NULL,
    `repost_of_id` CHAR(32) DEFAULT NULL,
    PRIMARY KEY (`id`),
    KEY `fk_posts_poster` (`poster_id`),
    KEY `fk_posts_reply_to` (`reply_to_id`),
    KEY `fk_posts_repost_of` (`repost_of_id`),
    CONSTRAINT `fk_posts_poster` FOREIGN KEY (`poster_id`) REFERENCES `users` (`id`),
    CONSTRAINT `fk_posts_reply_to` FOREIGN KEY (`reply_to_id`) REFERENCES `posts` (`id`),
    CONSTRAINT `fk_posts_repost_of` FOREIGN KEY (`repost_of_id`) REFERENCES `posts` (`id`)
);

CREATE TABLE `uploaded_files` (
    `id` CHAR(32) NOT NULL,
    `file_ext` varchar(128) NOT NULL,
    `created_at` datetime(6) NOT NULL,
    `uploaded_by_id` CHAR(32) NOT NULL,
    PRIMARY KEY (`id`),
    KEY `fk_uploaded_files_uploaded_by` (`uploaded_by_id`),
    CONSTRAINT `fk_uploaded_files_uploaded_by` FOREIGN KEY (`uploaded_by_id`) REFERENCES `users` (`id`)
);

ALTER TABLE
    `users`
ADD
    CONSTRAINT `fk_users_avatar` FOREIGN KEY (`avatar_id`) REFERENCES `uploaded_files` (`id`);

CREATE TABLE `user_follows` (
    `id` bigint(20) unsigned NOT NULL AUTO_INCREMENT,
    `follower_id` CHAR(32) NOT NULL,
    `followee_id` CHAR(32) NOT NULL,
    `created_at` datetime(6) DEFAULT NULL,
    PRIMARY KEY (`id`),
    UNIQUE KEY `idx_follower_followee` (`follower_id`, `followee_id`),
    KEY `fk_users_followers` (`followee_id`),
    CONSTRAINT `fk_users_followers` FOREIGN KEY (`followee_id`) REFERENCES `users` (`id`),
    CONSTRAINT `fk_users_following` FOREIGN KEY (`follower_id`) REFERENCES `users` (`id`)
);

CREATE TABLE `user_labels` (
    `user_id` CHAR(32) NOT NULL,
    `order` bigint(20) NOT NULL,
    `key` text NOT NULL,
    `value` text NOT NULL,
    PRIMARY KEY (`user_id`, `order`),
    CONSTRAINT `fk_users_user_labels` FOREIGN KEY (`user_id`) REFERENCES `users` (`id`)
);

CREATE TABLE `user_tokens` (
    `id` bigint(20) unsigned NOT NULL AUTO_INCREMENT,
    `user_id` CHAR(32) DEFAULT NULL,
    `token` varchar(64) NOT NULL,
    `created_at` datetime(6) NOT NULL,
    `last_used_at` datetime(6) NOT NULL,
    PRIMARY KEY (`id`),
    KEY `fk_users_user_tokens` (`user_id`),
    CONSTRAINT `fk_users_user_tokens` FOREIGN KEY (`user_id`) REFERENCES `users` (`id`)
);

CREATE TABLE `post_attachments` (
    `id` CHAR(32) NOT NULL,
    `post_id` CHAR(32) DEFAULT NULL,
    `uploaded_file_id` CHAR(32) NOT NULL,
    PRIMARY KEY (`id`),
    KEY `fk_post_attachments_uploaded_file` (`uploaded_file_id`),
    KEY `fk_post_attachments_post` (`post_id`),
    CONSTRAINT `fk_post_attachments_post` FOREIGN KEY (`post_id`) REFERENCES `posts` (`id`),
    CONSTRAINT `fk_post_attachments_uploaded_file` FOREIGN KEY (`uploaded_file_id`) REFERENCES `uploaded_files` (`id`)
);

CREATE TABLE `post_favorites` (
    `id` bigint(20) unsigned NOT NULL AUTO_INCREMENT,
    `post_id` CHAR(32) NOT NULL,
    `user_id` CHAR(32) NOT NULL,
    `is_bookmark` tinyint(1) NOT NULL,
    `created_at` datetime(6) DEFAULT NULL,
    PRIMARY KEY (`id`),
    UNIQUE KEY `idx_post_favorite_unique` (`post_id`, `user_id`, `is_bookmark`),
    KEY `fk_post_favorites_user` (`user_id`),
    CONSTRAINT `fk_post_favorites_post` FOREIGN KEY (`post_id`) REFERENCES `posts` (`id`),
    CONSTRAINT `fk_post_favorites_user` FOREIGN KEY (`user_id`) REFERENCES `users` (`id`)
);

CREATE TABLE `post_hashtags` (
    `id` bigint(20) unsigned NOT NULL AUTO_INCREMENT,
    `post_id` CHAR(32) NOT NULL,
    `hashtag_name` longtext NOT NULL,
    PRIMARY KEY (`id`),
    KEY `fk_posts_hashtags` (`post_id`),
    CONSTRAINT `fk_posts_hashtags` FOREIGN KEY (`post_id`) REFERENCES `posts` (`id`)
);

CREATE TABLE `post_mentions` (
    `id` bigint(20) unsigned NOT NULL AUTO_INCREMENT,
    `post_id` CHAR(32) NOT NULL,
    `target_user_id` CHAR(32) NOT NULL,
    PRIMARY KEY (`id`),
    KEY `fk_post_mentions_target_user` (`target_user_id`),
    KEY `fk_posts_mentions` (`post_id`),
    CONSTRAINT `fk_post_mentions_target_user` FOREIGN KEY (`target_user_id`) REFERENCES `users` (`id`),
    CONSTRAINT `fk_posts_mentions` FOREIGN KEY (`post_id`) REFERENCES `posts` (`id`)
);

create table `reactions` (
    `id` bigint unsigned not null PRIMARY KEY auto_increment,
    `name` varchar(128) not null
);

CREATE TABLE `post_reactions` (
    `id` bigint(20) unsigned NOT NULL PRIMARY KEY AUTO_INCREMENT,
    `post_id` CHAR(32) NOT NULL,
    `reaction_id` bigint unsigned NOT NULL,
    `user_id` CHAR(32) NOT NULL,
    `created_at` datetime(6) DEFAULT NULL,
    KEY `fk_post_reactions_post` (`post_id`),
    KEY `fk_post_reactions_user` (`user_id`),
    CONSTRAINT `fk_post_reactions_post` FOREIGN KEY (`post_id`) REFERENCES `posts` (`id`),
    CONSTRAINT `fk_post_reactions_user` FOREIGN KEY (`user_id`) REFERENCES `users` (`id`),
    CONSTRAINT `fk_post_reactions_reaction` FOREIGN KEY (`reaction_id`) REFERENCES `reactions` (`id`)
);

ALTER TABLE
    `users`
MODIFY
    COLUMN `avatar_id` CHAR(32) NULL DEFAULT NULL;

ALTER TABLE
    post_hashtags DROP FOREIGN KEY fk_posts_hashtags;

ALTER TABLE
    post_hashtags
ADD
    CONSTRAINT fk_posts_hashtags FOREIGN KEY (post_id) REFERENCES posts(id) ON DELETE CASCADE ON UPDATE RESTRICT;

ALTER TABLE
    user_tokens DROP FOREIGN KEY fk_users_user_tokens;

ALTER TABLE
    user_tokens
ADD
    CONSTRAINT fk_users_user_tokens FOREIGN KEY (user_id) REFERENCES users(id) ON DELETE CASCADE ON UPDATE RESTRICT;

ALTER TABLE
    `users`
ADD
    COLUMN `shared_inbox` VARCHAR(512) NULL DEFAULT NULL
AFTER
    `url`;

CREATE TABLE user_follow_requests(
    id CHAR(32) NOT NULL PRIMARY KEY,
    uri VARCHAR(512) NOT NULL UNIQUE,
    incoming TINYINT(1) NOT NULL,
    follower_id CHAR(32) NOT NULL,
    followee_id CHAR(32) NOT NULL,
    created_at DATETIME(6) NOT NULL DEFAULT CURRENT_TIMESTAMP(6),
    CONSTRAINT fk_users_follow_request_follower FOREIGN KEY (follower_id) REFERENCES users(id),
    CONSTRAINT fk_users_follow_request_followee FOREIGN KEY (followee_id) REFERENCES users(id)
);

CREATE TABLE `user_keys` (
    id VARCHAR(512) NOT NULL PRIMARY KEY,
    owner_id CHAR(32) NOT NULL,
    public_key TEXT NOT NULL,
    updated_at DATETIME(6) NOT NULL DEFAULT CURRENT_TIMESTAMP(6),
    CONSTRAINT fk_users_key_owner FOREIGN KEY (owner_id) REFERENCES users(id)
);

CREATE TABLE remote_users(
    user_id CHAR(32) NOT NULL PRIMARY KEY,
    `following` VARCHAR(512) NULL DEFAULT NULL,
    followers VARCHAR(512) NULL DEFAULT NULL,
    liked VARCHAR(512) NULL DEFAULT NULL,
    fetched_at DATETIME(6) NOT NULL DEFAULT CURRENT_TIMESTAMP(6),
    CONSTRAINT fk_users_remote_user FOREIGN KEY (user_id) REFERENCES users(id)
);

ALTER TABLE
    `users` RENAME COLUMN `url` to `uri`;

ALTER TABLE
    `posts`
ADD
    COLUMN `uri` VARCHAR(512) NULL DEFAULT NULL
AFTER
    `repost_of_id`;

ALTER TABLE
    `user_follow_requests`
MODIFY
    `uri` VARCHAR(512) NULL UNIQUE;

ALTER TABLE
    users
MODIFY
    COLUMN bpasswd varchar(60) NULL;

ALTER TABLE
    users
MODIFY
    COLUMN bio text DEFAULT '' NOT NULL;

ALTER TABLE
    users
MODIFY
    COLUMN created_at datetime(6) DEFAULT current_timestamp NOT NULL;

ALTER TABLE
    user_tokens
MODIFY
    COLUMN created_at datetime(6) DEFAULT current_timestamp(6) NOT NULL;

ALTER TABLE
    user_tokens
MODIFY
    COLUMN last_used_at datetime(6) DEFAULT current_timestamp(6) NOT NULL;

ALTER TABLE
    posts
MODIFY
    COLUMN inserted_at datetime(6) DEFAULT current_timestamp(6) NOT NULL;

ALTER TABLE
    user_follows
MODIFY
    COLUMN created_at datetime(6) DEFAULT current_timestamp(6) NOT NULL;

ALTER TABLE
    user_follows DROP FOREIGN KEY fk_users_followers;

ALTER TABLE
    user_follows
ADD
    CONSTRAINT fk_users_followers FOREIGN KEY (followee_id) REFERENCES users(id) ON DELETE CASCADE ON UPDATE RESTRICT;

ALTER TABLE
    user_follows DROP FOREIGN KEY fk_users_following;

ALTER TABLE
    user_follows
ADD
    CONSTRAINT fk_users_following FOREIGN KEY (follower_id) REFERENCES users(id) ON DELETE CASCADE ON UPDATE RESTRICT;

ALTER TABLE
    user_follow_requests DROP FOREIGN KEY fk_users_follow_request_followee;

ALTER TABLE
    user_follow_requests
ADD
    CONSTRAINT fk_users_follow_request_followee FOREIGN KEY (followee_id) REFERENCES users(id) ON DELETE CASCADE ON UPDATE RESTRICT;

ALTER TABLE
    user_follow_requests DROP FOREIGN KEY fk_users_follow_request_follower;

ALTER TABLE
    user_follow_requests
ADD
    CONSTRAINT fk_users_follow_request_follower FOREIGN KEY (follower_id) REFERENCES users(id) ON DELETE CASCADE ON UPDATE RESTRICT;

-- lightpub.remote_user_details definition
CREATE TABLE `remote_user_details` (
    `id` char(32) NOT NULL,
    `following_uri` varchar(512) DEFAULT NULL,
    `followers_uri` varchar(512) DEFAULT NULL,
    `liked_uri` varchar(512) DEFAULT NULL,
    `fetched_at` datetime(6) NOT NULL DEFAULT current_timestamp(6),
    PRIMARY KEY (`id`)
);

ALTER TABLE
    remote_user_details
ADD
    CONSTRAINT remote_user_details_users_FK FOREIGN KEY (id) REFERENCES users(id) ON DELETE CASCADE ON UPDATE CASCADE;

DROP TABLE remote_user_details;

ALTER TABLE
    remote_users DROP FOREIGN KEY fk_users_remote_user;

ALTER TABLE
    remote_users
ADD
    CONSTRAINT fk_users_remote_user FOREIGN KEY (user_id) REFERENCES users(id) ON DELETE CASCADE ON UPDATE CASCADE;

ALTER TABLE
    users DROP KEY idx_users_username;

ALTER TABLE
    user_follow_requests
ADD
    CONSTRAINT user_follow_requests_unique UNIQUE KEY (follower_id, followee_id);

ALTER TABLE
    posts DROP FOREIGN KEY fk_posts_reply_to;

ALTER TABLE
    posts
ADD
    CONSTRAINT fk_posts_reply_to FOREIGN KEY (reply_to_id) REFERENCES posts(id) ON DELETE CASCADE ON UPDATE CASCADE;

ALTER TABLE
    posts DROP FOREIGN KEY fk_posts_repost_of;

ALTER TABLE
    posts
ADD
    CONSTRAINT fk_posts_repost_of FOREIGN KEY (repost_of_id) REFERENCES posts(id) ON DELETE CASCADE ON UPDATE CASCADE;

ALTER TABLE
    posts DROP FOREIGN KEY fk_posts_poster;

ALTER TABLE
    posts
ADD
    CONSTRAINT fk_posts_poster FOREIGN KEY (poster_id) REFERENCES users(id) ON DELETE CASCADE ON UPDATE CASCADE;

ALTER TABLE
    users
ADD
    CONSTRAINT users_uri_unique UNIQUE KEY (uri);

ALTER TABLE
    users DROP CONSTRAINT users_uri_unique;

ALTER TABLE
    uploaded_files
MODIFY
    COLUMN created_at datetime(6) DEFAULT CURRENT_TIMESTAMP(6) NOT NULL;

ALTER TABLE
    post_favorites
MODIFY
    COLUMN created_at datetime(6) DEFAULT CURRENT_TIMESTAMP(6) NOT NULL;

ALTER TABLE
    post_reactions
MODIFY
    COLUMN created_at datetime(6) DEFAULT CURRENT_TIMESTAMP(6) NOT NULL;

ALTER TABLE
    post_favorites
MODIFY
    COLUMN id CHAR(32) NOT NULL;

ALTER TABLE
    post_reactions
MODIFY
    COLUMN id CHAR(32) NOT NULL;

ALTER TABLE
    post_reactions CHANGE reaction_id custom_reaction_id bigint(20) unsigned NULL;

ALTER TABLE
    post_reactions
MODIFY
    COLUMN custom_reaction_id bigint(20) unsigned NULL DEFAULT NULL;

ALTER TABLE
    post_reactions
ADD
    reaction_str varchar(255) DEFAULT NULL NULL;

ALTER TABLE
    post_reactions CHANGE reaction_str reaction_str varchar(255) DEFAULT NULL NULL
AFTER
    post_id;

ALTER TABLE
    post_reactions COLLATE = utf8mb4_bin;