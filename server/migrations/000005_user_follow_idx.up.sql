CREATE UNIQUE INDEX `user_follow_unique_idx` ON UserFollow(followerId, followeeId);

CREATE INDEX `user_follow_follower_idx` ON UserFollow(followerId);