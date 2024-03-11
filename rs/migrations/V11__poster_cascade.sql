ALTER TABLE
    posts DROP FOREIGN KEY fk_posts_reply_to;

ALTER TABLE
    posts
ADD
    CONSTRAINT fk_posts_reply_to FOREIGN KEY (reply_to_id) REFERENCES posts(id) ON DELETE CASCADE ON UPDATE CASCADE;

ALTER TABLE
    posts DROP FOREIGN KEY fk_posts_repost_of;

ALTER TABLE
    posts
ADD
    CONSTRAINT fk_posts_repost_of FOREIGN KEY (repost_of_id) REFERENCES posts(id) ON DELETE CASCADE ON UPDATE CASCADE;

ALTER TABLE
    posts DROP FOREIGN KEY fk_posts_poster;

ALTER TABLE
    posts
ADD
    CONSTRAINT fk_posts_poster FOREIGN KEY (poster_id) REFERENCES users(id) ON DELETE CASCADE ON UPDATE CASCADE;