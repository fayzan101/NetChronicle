use chrono::{DateTime, Utc};
use netchronicle_common::ActivityCategory;
use sqlx::PgPool;
use uuid::Uuid;

use crate::models::{category_to_db, AppActivityRow, WebsiteLogRow};

pub struct ActivityRepository<'a> {
    pool: &'a PgPool,
}

impl<'a> ActivityRepository<'a> {
    pub fn new(pool: &'a PgPool) -> Self {
        Self { pool }
    }

    pub async fn insert_app_log(
        &self,
        user_id: Uuid,
        app_name: &str,
        window_title: Option<&str>,
        duration_sec: i32,
        category: ActivityCategory,
        recorded_at: DateTime<Utc>,
    ) -> anyhow::Result<Uuid> {
        let id = sqlx::query_scalar::<_, Uuid>(
            r#"
            INSERT INTO app_activity_logs
                (user_id, app_name, window_title, duration_sec, category, recorded_at)
            VALUES ($1, $2, $3, $4, $5::activity_category, $6)
            RETURNING id
            "#,
        )
        .bind(user_id)
        .bind(app_name)
        .bind(window_title)
        .bind(duration_sec)
        .bind(category_to_db(category))
        .bind(recorded_at)
        .fetch_one(self.pool)
        .await?;

        Ok(id)
    }

    pub async fn insert_website_log(
        &self,
        user_id: Uuid,
        url: &str,
        domain: &str,
        time_spent_sec: i32,
        category: ActivityCategory,
        visited_at: DateTime<Utc>,
    ) -> anyhow::Result<Uuid> {
        let id = sqlx::query_scalar::<_, Uuid>(
            r#"
            INSERT INTO website_logs
                (user_id, url, domain, time_spent_sec, category, visited_at)
            VALUES ($1, $2, $3, $4, $5::activity_category, $6)
            RETURNING id
            "#,
        )
        .bind(user_id)
        .bind(url)
        .bind(domain)
        .bind(time_spent_sec)
        .bind(category_to_db(category))
        .bind(visited_at)
        .fetch_one(self.pool)
        .await?;

        Ok(id)
    }

    pub async fn insert_raw_event(
        &self,
        user_id: Uuid,
        event_type: &str,
        payload: serde_json::Value,
        recorded_at: DateTime<Utc>,
    ) -> anyhow::Result<()> {
        sqlx::query(
            r#"
            INSERT INTO raw_events (user_id, event_type, payload, recorded_at)
            VALUES ($1, $2, $3, $4)
            "#,
        )
        .bind(user_id)
        .bind(event_type)
        .bind(payload)
        .bind(recorded_at)
        .execute(self.pool)
        .await?;

        Ok(())
    }

    pub async fn list_app_logs(
        &self,
        user_id: Uuid,
        from: DateTime<Utc>,
        to: DateTime<Utc>,
        limit: i64,
        offset: i64,
    ) -> anyhow::Result<Vec<AppActivityRow>> {
        let rows = sqlx::query_as::<_, AppActivityRow>(
            r#"
            SELECT id, user_id, session_id, app_name, window_title, duration_sec,
                   category::text AS category, recorded_at
            FROM app_activity_logs
            WHERE user_id = $1 AND recorded_at >= $2 AND recorded_at < $3
            ORDER BY recorded_at ASC
            LIMIT $4 OFFSET $5
            "#,
        )
        .bind(user_id)
        .bind(from)
        .bind(to)
        .bind(limit)
        .bind(offset)
        .fetch_all(self.pool)
        .await?;

        Ok(rows)
    }

    pub async fn list_website_logs(
        &self,
        user_id: Uuid,
        from: DateTime<Utc>,
        to: DateTime<Utc>,
        limit: i64,
        offset: i64,
    ) -> anyhow::Result<Vec<WebsiteLogRow>> {
        let rows = sqlx::query_as::<_, WebsiteLogRow>(
            r#"
            SELECT id, url, domain, time_spent_sec, category::text AS category, visited_at
            FROM website_logs
            WHERE user_id = $1 AND visited_at >= $2 AND visited_at < $3
            ORDER BY visited_at ASC
            LIMIT $4 OFFSET $5
            "#,
        )
        .bind(user_id)
        .bind(from)
        .bind(to)
        .bind(limit)
        .bind(offset)
        .fetch_all(self.pool)
        .await?;

        Ok(rows)
    }

    pub async fn latest_snapshot(
        &self,
        user_id: Uuid,
    ) -> anyhow::Result<Option<crate::models::ActivitySnapshotRow>> {
        let row = sqlx::query_as::<_, crate::models::ActivitySnapshotRow>(
            r#"
            SELECT payload, recorded_at
            FROM raw_events
            WHERE user_id = $1 AND event_type = 'activity_snapshot'
            ORDER BY recorded_at DESC
            LIMIT 1
            "#,
        )
        .bind(user_id)
        .fetch_optional(self.pool)
        .await?;

        Ok(row)
    }

    pub async fn latest_browser_tab(
        &self,
        user_id: Uuid,
    ) -> anyhow::Result<Option<crate::models::ActivitySnapshotRow>> {
        let row = sqlx::query_as::<_, crate::models::ActivitySnapshotRow>(
            r#"
            SELECT payload, recorded_at
            FROM raw_events
            WHERE user_id = $1 AND event_type = 'browser_tab'
            ORDER BY recorded_at DESC
            LIMIT 1
            "#,
        )
        .bind(user_id)
        .fetch_optional(self.pool)
        .await?;

        Ok(row)
    }
}
