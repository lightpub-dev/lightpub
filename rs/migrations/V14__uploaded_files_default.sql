ALTER TABLE
    uploaded_files
MODIFY
    COLUMN created_at datetime(6) DEFAULT CURRENT_TIMESTAMP NOT NULL;