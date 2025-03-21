DROP PROCEDURE IF EXISTS `get_note_ids_generalized`;

CREATE PROCEDURE `get_note_ids_generalized`(
    IN viewer_id BINARY(16),
    IN include_self BOOLEAN,
    IN include_public BOOLEAN,
    IN include_unlisted BOOLEAN,
    IN limit_reply_to_id BINARY(16),
    IN lim BIGINT,
    IN before_date DATETIME
) BEGIN
SELECT
    n.id
FROM
    note n
WHERE
    -- visibility check
    (
        -- self notes
        (
            CASE
                WHEN include_self
                AND (viewer_id IS NOT NULL) THEN n.author_id = viewer_id
                ELSE FALSE
            END
        ) -- public notes
        OR (
            CASE
                WHEN include_public THEN n.visibility = 'public'
                ELSE FALSE
            END
        ) -- unlisted notes
        OR (
            CASE
                WHEN include_unlisted THEN n.visibility = 'unlisted'
                ELSE FALSE
            END
        ) -- follower notes
        OR (
            CASE
                WHEN viewer_id IS NULL THEN FALSE
                ELSE (
                    (
                        n.visibility IN ('public', 'unlisted', 'follower')
                    )
                    AND (
                        EXISTS (
                            SELECT
                                f.id
                            FROM
                                user_follow f
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
                            note_mention m
                        WHERE
                            m.target_user_id = viewer_id
                            AND m.note_id = n.id
                    )
                )
            END
        )
    ) -- deleted_at
    AND (n.deleted_at IS NULL) -- limit to replies
    AND (
        CASE
            WHEN limit_reply_to_id IS NULL THEN TRUE
            ELSE n.reply_to_id = limit_reply_to_id
        END
    ) -- limit before_date
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