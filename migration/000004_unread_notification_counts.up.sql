CREATE VIEW unread_notification_counts AS
SELECT
    user_id,
    COUNT(*) AS unread_count
FROM
    notifications
WHERE
    read_at IS NULL
GROUP BY
    user_id;