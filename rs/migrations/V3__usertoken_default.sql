ALTER TABLE
    user_tokens
MODIFY
    COLUMN last_used_at datetime(6) DEFAULT current_timestamp NOT NULL;