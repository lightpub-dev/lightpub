use sea_orm_migration::prelude::*;

#[derive(DeriveMigrationName)]
pub struct Migration;

#[async_trait::async_trait]
impl MigrationTrait for Migration {
    async fn up(&self, manager: &SchemaManager) -> Result<(), DbErr> {
        // Replace the sample below with your own migration scripts
        let db = manager.get_connection();

        db.execute_unprepared(
            "
CREATE VIEW trending_tags AS
SELECT nt.tag_id, t.name, COUNT(n.id) AS note_count
FROM note_tag nt
INNER JOIN note n ON n.id = nt.note_id
INNER JOIN tag t ON t.id = nt.tag_id
WHERE n.created_at >= NOW() - INTERVAL '1 day' AND n.visibility IN ('public', 'unlisted')
GROUP BY nt.tag_id, t.name
ORDER BY note_count DESC;
            ",
        )
        .await?;

        Ok(())
    }

    async fn down(&self, manager: &SchemaManager) -> Result<(), DbErr> {
        // Replace the sample below with your own migration scripts
        let db = manager.get_connection();

        db.execute_unprepared("DROP VIEW trending_tags").await?;

        Ok(())
    }
}
