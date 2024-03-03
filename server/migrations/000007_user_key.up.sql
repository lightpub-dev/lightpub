CREATE TABLE `user_keys` (
    id VARCHAR(512) NOT NULL PRIMARY KEY,
    owner_id VARCHAR(32) NOT NULL,
    public_key TEXT NOT NULL,
    updated_at DATETIME(6) NOT NULL DEFAULT CURRENT_TIMESTAMP(6),
    CONSTRAINT fk_users_key_owner FOREIGN KEY (owner_id) REFERENCES users(id)
);