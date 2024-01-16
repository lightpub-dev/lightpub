ALTER TABLE
    User CHANGE id id BINARY(16) NOT NULL;

ALTER TABLE
    UserFollow CHANGE follower_id follower_id BINARY(16) NOT NULL;

ALTER TABLE
    UserFollow CHANGE followee_id followee_id BINARY(16) NOT NULL;

ALTER TABLE
    UserToken CHANGE user_id user_id BINARY(16) NOT NULL;

ALTER TABLE
    PollChoice CHANGE poll_id poll_id BINARY(16) NOT NULL;

ALTER TABLE
    PollVote CHANGE poll_id poll_id BINARY(16) NOT NULL;

ALTER TABLE
    PollVote CHANGE user_id user_id BINARY(16) NOT NULL;

ALTER TABLE
    Post CHANGE id id BINARY(16) NOT NULL;

ALTER TABLE
    Post CHANGE poster_id poster_id BINARY(16) NOT NULL;

ALTER TABLE
    Post CHANGE reply_to reply_to BINARY(16) NULL;

ALTER TABLE
    Post CHANGE repost_of repost_of BINARY(16) NULL;

ALTER TABLE
    Post CHANGE poll_id poll_id BINARY(16) NULL;

ALTER TABLE
    PostAttachment CHANGE id id BINARY(16) NOT NULL;

ALTER TABLE
    PostAttachment CHANGE post_id post_id BINARY(16) NOT NULL;

ALTER TABLE
    PostFavorite CHANGE post_id post_id BINARY(16) NOT NULL;

ALTER TABLE
    PostFavorite CHANGE user_id user_id BINARY(16) NOT NULL;

ALTER TABLE
    PostHashtag CHANGE post_id post_id BINARY(16) NOT NULL;

ALTER TABLE
    PostMention CHANGE post_id post_id BINARY(16) NOT NULL;

ALTER TABLE
    PostMention CHANGE target_user_id target_user_id BINARY(16) NOT NULL;

ALTER TABLE
    PostPoll CHANGE id id BINARY(16) NOT NULL;

ALTER TABLE
    PostReaction CHANGE post_id post_id BINARY(16) NOT NULL;

ALTER TABLE
    PostReaction CHANGE user_id user_id BINARY(16) NOT NULL;