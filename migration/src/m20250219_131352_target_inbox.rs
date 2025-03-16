use sea_orm_migration::prelude::*;

#[derive(DeriveMigrationName)]
pub struct Migration;

#[async_trait::async_trait]
impl MigrationTrait for Migration {
    async fn up(&self, manager: &SchemaManager) -> Result<(), DbErr> {
        // Replace the sample below with your own migration scripts
        let db = manager.get_connection();

        db.execute_unprepared(r#"
        ALTER TABLE IF EXISTS public."user"
            ADD COLUMN preferred_inbox character varying(512) GENERATED ALWAYS AS (COALESCE(shared_inbox, inbox)) STORED;
        "#).await?;

        db.execute_unprepared(
            r#"
    CREATE OR REPLACE FUNCTION public.find_target_inboxes(
	note_id uuid,
	include_author boolean)
                RETURNS TABLE(inbox text)
                LANGUAGE 'sql'
                COST 100
                VOLATILE PARALLEL UNSAFE
                ROWS 1000

            AS $BODY$
            select DISTINCT ON (u.preferred_inbox) u.preferred_inbox from
            (
            -- self
            (
            SELECT (CASE
	WHEN include_author THEN (SELECT author_id FROM note WHERE id=note_id)
	ELSE NULL
            END) AS user_id
            )
            UNION
            -- followers
            (
            SELECT
	(CASE
	WHEN ((SELECT visibility FROM note WHERE id=note_id) IN ('public','unlisted','follower'))
		 THEN f.follower_id
	ELSE NULL
	END) AS user_id
            FROM user_follow f
            WHERE f.followed_id = (SELECT author_id FROM note WHERE id=note_id)
              AND f.pending = FALSE
            )
            UNION
            -- mentioned
            (
            SELECT m.target_user_id AS user_id
            FROM note_mention m
            WHERE m.note_id = note_id
            )
            ) ui
            INNER JOIN "user" u ON ui.user_id=u.id
            WHERE u.preferred_inbox IS NOT NULL
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
        DROP FUNCTION IF EXISTS public.find_target_inboxes(uuid, boolean);
        "#,
        )
        .await?;

        db.execute_unprepared(
            r#"
        ALTER TABLE IF EXISTS public."user" DROP COLUMN IF EXISTS preferred_inbox;
        "#,
        )
        .await?;

        Ok(())
    }
}
