CREATE VIEW actual_user_follows AS
SELECT
    id,
    follower_id,
    followed_id,
    url,
    created_at
FROM
    user_follows
WHERE
    NOT pending;