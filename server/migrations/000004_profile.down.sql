DROP TABLE UserProfile;

DROP TABLE UserLabel;

ALTER TABLE
    User
ADD
    COLUMN bio TEXT NOT NULL
AFTER
    outbox;