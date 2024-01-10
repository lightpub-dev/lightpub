CREATE UNIQUE INDEX `user_follow_unique_idx` ON UserFollow(follower_id, followee_id);

CREATE INDEX `user_follow_follower_idx` ON UserFollow(follower_id);
