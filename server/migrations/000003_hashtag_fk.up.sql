ALTER TABLE
    lightpub.post_hashtags DROP FOREIGN KEY fk_posts_hashtags;

ALTER TABLE
    lightpub.post_hashtags
ADD
    CONSTRAINT fk_posts_hashtags FOREIGN KEY (post_id) REFERENCES lightpub.posts(id) ON DELETE CASCADE ON UPDATE RESTRICT;