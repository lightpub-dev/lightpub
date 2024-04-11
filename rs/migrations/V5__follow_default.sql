ALTER TABLE
    user_follows
MODIFY
    COLUMN created_at datetime(6) DEFAULT current_timestamp(6) NOT NULL;