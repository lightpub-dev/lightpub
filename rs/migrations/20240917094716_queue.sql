-- Add migration script here
CREATE TABLE QueuedTask(
    id BIGINT NOT NULL PRIMARY KEY AUTO_INCREMENT,
    created_at DATETIME NOT NULL DEFAULT CURRENT_TIMESTAMP,
    started_at DATETIME DEFAULT NULL,
    current_retry INTEGER NOT NULL DEFAULT 0,
    max_retry INTEGER NOT NULL,
    payload TEXT NOT NULL
);