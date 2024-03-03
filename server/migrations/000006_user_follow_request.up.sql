CREATE TABLE user_follow_requests(
    id VARCHAR(32) NOT NULL PRIMARY KEY,
    uri VARCHAR(512) NOT NULL UNIQUE,
    incoming TINYINT(1) NOT NULL,
    follower_id VARCHAR(32) NOT NULL,
    followee_id VARCHAR(32) NOT NULL,
    created_at DATETIME(6) NOT NULL DEFAULT CURRENT_TIMESTAMP(6),
    CONSTRAINT fk_users_follow_request_follower FOREIGN KEY (follower_id) REFERENCES users(id),
    CONSTRAINT fk_users_follow_request_followee FOREIGN KEY (followee_id) REFERENCES users(id)
);