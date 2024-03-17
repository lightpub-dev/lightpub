ALTER TABLE
    posts
MODIFY
    COLUMN inserted_at datetime(6) DEFAULT current_timestamp NOT NULL;