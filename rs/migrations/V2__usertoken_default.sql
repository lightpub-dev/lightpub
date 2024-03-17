ALTER TABLE
    user_tokens
MODIFY
    COLUMN created_at datetime(6) DEFAULT current_timestamp NOT NULL;