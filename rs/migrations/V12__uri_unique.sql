ALTER TABLE
    users
ADD
    CONSTRAINT users_uri_unique UNIQUE KEY (uri);