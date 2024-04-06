ALTER TABLE
    post_favorites
MODIFY
    COLUMN id CHAR(32) NOT NULL;

ALTER TABLE
    post_reactions
MODIFY
    COLUMN id CHAR(32) NOT NULL;