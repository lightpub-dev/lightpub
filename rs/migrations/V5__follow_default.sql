ALTER TABLE
    lightpub.user_follows
MODIFY
    COLUMN created_at datetime(6) DEFAULT current_timestamp NOT NULL;