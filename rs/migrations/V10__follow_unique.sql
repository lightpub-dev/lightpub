ALTER TABLE
    user_follow_requests
ADD
    CONSTRAINT user_follow_requests_unique UNIQUE KEY (follower_id, followee_id);