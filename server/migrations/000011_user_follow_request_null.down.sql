ALTER TABLE
    `user_follow_requests`
MODIFY
    `uri` VARCHAR(512) NOT NULL UNIQUE;