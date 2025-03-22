DROP PROCEDURE IF EXISTS `get_user_note_ids`;

CREATE PROCEDURE `get_user_note_ids`(
    IN viewer_id BINARY(16),
    IN target_user_id BINARY(16),
    IN lim BIGINT,
    IN before_date DATETIME
) BEGIN
SELECT
    n.id
FROM
    notes n
WHERE
    -- target user
    (n.author_id = target_user_id) -- visibility check
    AND (
        -- self notes
        (
            CASE
                WHEN viewer_id IS NOT NULL THEN target_user_id = viewer_id
                ELSE FALSE
            END
        ) -- public or unlisted notes
        OR n.visibility IN ('public', 'unlisted') -- unlisted notes
        -- follower notes
        OR (
            CASE
                WHEN viewer_id IS NULL THEN FALSE
                ELSE (
                    (
                        n.visibility = 'follower'
                    )
                    AND (
                        EXISTS (
                            SELECT
                                f.id
                            FROM
                                user_follows f
                            WHERE
                                f.follower_id = viewer_id
                                AND f.followed_id = n.author_id
                                AND f.pending = FALSE
                        )
                    )
                )
            END
        ) -- mentioned notes
        OR (
            CASE
                WHEN viewer_id IS NULL THEN FALSE
                ELSE (
                    EXISTS (
                        SELECT
                            m.id
                        FROM
                            note_mentions m
                        WHERE
                            m.target_user_id = viewer_id
                            AND m.note_id = n.id
                    )
                )
            END
        )
    ) -- deleted_at
    AND (n.deleted_at IS NULL) -- limit before_date
    AND (
        CASE
            WHEN before_date IS NULL THEN TRUE
            ELSE n.created_at <= before_date
        END
    )
ORDER BY
    n.created_at DESC
LIMIT
    lim;

END