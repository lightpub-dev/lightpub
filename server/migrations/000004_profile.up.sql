ALTER TABLE
    User DROP COLUMN bio;

CREATE TABLE `UserLabel` (
    `user_id` BINARY(16) NOT NULL,
    `order` INT NOT NULL,
    `key` TEXT NOT NULL,
    `value` TEXT NOT NULL,
    PRIMARY KEY (`user_id`, `order`)
);

CREATE TABLE `UserProfile` (
    `user_id` binary(16) NOT NULL,
    `bio` text NOT NULL,
    PRIMARY KEY (`user_id`)
);