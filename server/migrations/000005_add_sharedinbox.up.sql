ALTER TABLE
    `users`
ADD
    COLUMN `shared_inbox` VARCHAR(512) NULL DEFAULT NULL
AFTER
    `url`;