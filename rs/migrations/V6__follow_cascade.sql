ALTER TABLE
    user_follows DROP FOREIGN KEY fk_users_followers;

ALTER TABLE
    user_follows
ADD
    CONSTRAINT fk_users_followers FOREIGN KEY (followee_id) REFERENCES users(id) ON DELETE CASCADE ON UPDATE RESTRICT;

ALTER TABLE
    user_follows DROP FOREIGN KEY fk_users_following;

ALTER TABLE
    user_follows
ADD
    CONSTRAINT fk_users_following FOREIGN KEY (follower_id) REFERENCES users(id) ON DELETE CASCADE ON UPDATE RESTRICT;

ALTER TABLE
    user_follow_requests DROP FOREIGN KEY fk_users_follow_request_followee;

ALTER TABLE
    user_follow_requests
ADD
    CONSTRAINT fk_users_follow_request_followee FOREIGN KEY (followee_id) REFERENCES users(id) ON DELETE CASCADE ON UPDATE RESTRICT;

ALTER TABLE
    user_follow_requests DROP FOREIGN KEY fk_users_follow_request_follower;

ALTER TABLE
    user_follow_requests
ADD
    CONSTRAINT fk_users_follow_request_follower FOREIGN KEY (follower_id) REFERENCES users(id) ON DELETE CASCADE ON UPDATE RESTRICT;