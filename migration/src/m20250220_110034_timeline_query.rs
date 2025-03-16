use sea_orm_migration::prelude::*;

#[derive(DeriveMigrationName)]
pub struct Migration;

#[async_trait::async_trait]
impl MigrationTrait for Migration {
    async fn up(&self, manager: &SchemaManager) -> Result<(), DbErr> {
        // Replace the sample below with your own migration scripts
        let db = manager.get_connection();

        db.execute_unprepared(
            r#"
        CREATE OR REPLACE FUNCTION public.get_note_ids_generalized(
	viewer_id uuid,
	include_public boolean,
	include_unlisted boolean,
	limit_reply_to_id uuid,
	lim bigint,
	before_date timestamp with time zone)
    RETURNS TABLE(id uuid) 
    LANGUAGE 'sql'
    COST 100
    VOLATILE PARALLEL UNSAFE
    ROWS 1000

AS $BODY$
SELECT n.id
FROM note n
WHERE
-- visibility check
(
	-- self notes
	(
	CASE
		WHEN false THEN FALSE
		ELSE n.author_id = '019522da-fca3-3c35-ef52-dda09bb5e172'
	END
	)
	-- public notes
	OR (
	CASE
		WHEN include_public THEN n.visibility = 'public'
		ELSE FALSE
	END
	)
	-- unlisted notes
	OR (
	CASE
		WHEN include_unlisted THEN n.visibility = 'unlisted'
		ELSE FALSE
	END
	)
	-- follower notes
	OR (
	CASE
		WHEN viewer_id IS NULL THEN FALSE
		ELSE (
			(n.visibility IN ('public', 'unlisted', 'follower'))
			AND (
				EXISTS (
					SELECT f.id
					FROM user_follow f
					WHERE f.follower_id = viewer_id
					  AND f.followed_id = n.author_id
					  AND f.pending = FALSE
				)
			)
		)
	END
	)
	-- mentioned notes
	OR (
	CASE
		WHEN viewer_id IS NULL THEN FALSE
		ELSE (
			EXISTS (
				SELECT m.id
				FROM note_mention m
				WHERE m.target_user_id = viewer_id
				  AND m.note_id = n.id
			)
		)
	END
	)
)
-- deleted_at
AND (
	n.deleted_at IS NULL
)
-- limit to replies
AND (
CASE
	WHEN limit_reply_to_id IS NULL THEN TRUE
	ELSE n.reply_to_id = limit_reply_to_id
END
)
-- limit before_date
AND (
CASE
	WHEN before_date IS NULL THEN TRUE
	ELSE n.created_at <= before_date
END
)
ORDER BY n.created_at DESC
LIMIT lim
$BODY$;
        "#,
        )
        .await?;

        Ok(())
    }

    async fn down(&self, manager: &SchemaManager) -> Result<(), DbErr> {
        // Replace the sample below with your own migration scripts
        let db = manager.get_connection();

        db.execute_unprepared(
            r#"
        DROP FUNCTION public.get_note_ids_generalized
        "#,
        )
        .await?;

        Ok(())
    }
}
