use chrono::{DateTime, NaiveDate, Utc};
use sqlx::PgPool;
use uuid::Uuid;

use crate::models::{CategoryBreakdownRow, DailyActivityStats};

pub struct AnalyticsRepository<'a> {
    pool: &'a PgPool,
}

impl<'a> AnalyticsRepository<'a> {
    pub fn new(pool: &'a PgPool) -> Self {
        Self { pool }
    }

    pub async fn daily_activity_stats(
        &self,
        user_id: Uuid,
        day: NaiveDate,
    ) -> anyhow::Result<DailyActivityStats> {
        let start = day.and_hms_opt(0, 0, 0).unwrap().and_utc();
        let end = (day + chrono::Duration::days(1))
            .and_hms_opt(0, 0, 0)
            .unwrap()
            .and_utc();

        let stats = sqlx::query_as::<_, DailyActivityStats>(
            r#"
            SELECT
                COALESCE(SUM(duration_sec), 0)::bigint AS total_sec,
                COALESCE(SUM(duration_sec) FILTER (WHERE category IN ('work', 'learning')), 0)::bigint AS productive_sec,
                COALESCE(SUM(duration_sec) FILTER (WHERE category = 'distraction'), 0)::bigint AS distraction_sec
            FROM app_activity_logs
            WHERE user_id = $1 AND recorded_at >= $2 AND recorded_at < $3
            "#,
        )
        .bind(user_id)
        .bind(start)
        .bind(end)
        .fetch_one(self.pool)
        .await?;

        Ok(stats)
    }

    pub async fn category_breakdown(
        &self,
        user_id: Uuid,
        from: DateTime<Utc>,
        to: DateTime<Utc>,
    ) -> anyhow::Result<Vec<CategoryBreakdownRow>> {
        let rows = sqlx::query_as::<_, CategoryBreakdownRow>(
            r#"
            SELECT category::text AS category, COALESCE(SUM(duration_sec), 0)::bigint AS total_sec
            FROM app_activity_logs
            WHERE user_id = $1 AND recorded_at >= $2 AND recorded_at < $3
            GROUP BY category
            ORDER BY total_sec DESC
            "#,
        )
        .bind(user_id)
        .bind(from)
        .bind(to)
        .fetch_all(self.pool)
        .await?;

        Ok(rows)
    }

    pub async fn top_apps(
        &self,
        user_id: Uuid,
        from: DateTime<Utc>,
        to: DateTime<Utc>,
        limit: i64,
    ) -> anyhow::Result<Vec<(String, i64)>> {
        let rows = sqlx::query_as::<_, (String, i64)>(
            r#"
            SELECT app_name, COALESCE(SUM(duration_sec), 0)::bigint AS total_sec
            FROM app_activity_logs
            WHERE user_id = $1 AND recorded_at >= $2 AND recorded_at < $3
            GROUP BY app_name
            ORDER BY total_sec DESC
            LIMIT $4
            "#,
        )
        .bind(user_id)
        .bind(from)
        .bind(to)
        .bind(limit)
        .fetch_all(self.pool)
        .await?;

        Ok(rows)
    }

    pub async fn top_domains(
        &self,
        user_id: Uuid,
        from: DateTime<Utc>,
        to: DateTime<Utc>,
        limit: i64,
    ) -> anyhow::Result<Vec<(String, i64)>> {
        let rows = sqlx::query_as::<_, (String, i64)>(
            r#"
            SELECT domain, COALESCE(SUM(time_spent_sec), 0)::bigint AS total_sec
            FROM website_logs
            WHERE user_id = $1 AND visited_at >= $2 AND visited_at < $3
            GROUP BY domain
            ORDER BY total_sec DESC
            LIMIT $4
            "#,
        )
        .bind(user_id)
        .bind(from)
        .bind(to)
        .bind(limit)
        .fetch_all(self.pool)
        .await?;

        Ok(rows)
    }
}
