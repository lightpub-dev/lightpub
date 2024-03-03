ALTER TABLE
    `posts`
ADD
    COLUMN `uri` VARCHAR(512) NULL DEFAULT NULL
AFTER
    `repost_of_id`;