use async_trait::async_trait;
use derive_more::Constructor;
use gen_span::gen_span;
use sqlx::SqlitePool;
use uuid::fmt::Simple;

use crate::backend::{
    GetTrendError, IDOnlyEntity, ServiceError, TrendEntry, TrendService, TrendingHashtag,
};

#[derive(Constructor)]
pub struct DBTrendService {
    pool: SqlitePool,
}

#[derive(Debug)]
struct HashtagCount {
    hashtag: String,
    count: i64,
}

#[gen_span]
#[async_trait]
impl TrendService for DBTrendService {
    async fn trending_hashtags(
        &self,
        top_trend_n: i64,
        top_posts_n: i64,
    ) -> Result<TrendingHashtag, ServiceError<GetTrendError>> {
        let since = { chrono::Utc::now() - chrono::Duration::try_hours(24).unwrap() }.naive_utc();
        let top_trends = sqlx::query_as!(
            HashtagCount,
            r#"
            SELECT h.hashtag_name AS hashtag, COUNT(p.id) AS count
            FROM post_hashtags h
            INNER JOIN posts p ON h.post_id=p.id
            WHERE p.created_at > ?
            GROUP BY h.hashtag_name
            ORDER BY count DESC
            LIMIT ?
            "#,
            since,
            top_trend_n
        )
        .fetch_all(&self.pool)
        .await?;

        let mut trend_entries: Vec<TrendEntry> = Vec::new();
        for trend in &top_trends {
            let posts = sqlx::query_as!(
                IDOnlyEntity,
                r#"
                SELECT p.id AS `id: Simple`
                FROM posts p
                INNER JOIN post_hashtags h ON p.id=h.post_id
                WHERE h.hashtag_name=?
                ORDER BY p.created_at DESC
                LIMIT ?
                "#,
                trend.hashtag,
                top_posts_n
            )
            .fetch_all(&self.pool)
            .await?;
            trend_entries.push(TrendEntry {
                hashtag: trend.hashtag.clone(),
                count: trend.count,
                posts,
            });
        }

        Ok(TrendingHashtag {
            trends: trend_entries,
        })
    }
}
