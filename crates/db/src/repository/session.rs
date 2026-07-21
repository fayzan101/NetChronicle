use chrono::{DateTime, NaiveDate, Utc};
use netchronicle_common::SessionDraft;
use sqlx::PgPool;
use uuid::Uuid;

use crate::models::{category_to_db, stability_to_db, SessionRow};

pub struct SessionRepository<'a> {
    pool: &'a PgPool,
}

impl<'a> SessionRepository<'a> {
    pub fn new(pool: &'a PgPool) -> Self {
        Self { pool }
    }

    pub async fn insert(&self, draft: &SessionDraft) -> anyhow::Result<Uuid> {
        let stability = draft.network_stability.map(stability_to_db);

        let id = sqlx::query_scalar::<_, Uuid>(
            r#"
            INSERT INTO sessions
                (user_id, start_time, end_time, category, productivity_score, network_stability, primary_apps)
            VALUES ($1, $2, $3, $4::activity_category, $5, $6::network_stability, $7)
            RETURNING session_id
            "#,
        )
        .bind(draft.user_id)
        .bind(draft.start_time)
        .bind(draft.end_time)
        .bind(category_to_db(draft.category))
        .bind(draft.productivity_score)
        .bind(stability)
        .bind(&draft.primary_apps)
        .fetch_one(self.pool)
        .await?;

        Ok(id)
    }

    pub async fn list(
        &self,
        user_id: Uuid,
        from: DateTime<Utc>,
        to: DateTime<Utc>,
        limit: i64,
        offset: i64,
    ) -> anyhow::Result<Vec<SessionRow>> {
        let rows = sqlx::query_as::<_, SessionRow>(
            r#"
            SELECT
                session_id, user_id, start_time, end_time,
                category::text AS category,
                productivity_score,
                network_stability::text AS network_stability,
                primary_apps
            FROM sessions
            WHERE user_id = $1 AND start_time >= $2 AND start_time < $3
            ORDER BY start_time DESC
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

    pub async fn clear_for_day(&self, user_id: Uuid, day: NaiveDate) -> anyhow::Result<()> {
        let start = day.and_hms_opt(0, 0, 0).unwrap().and_utc();
        let end = (day + chrono::Duration::days(1))
            .and_hms_opt(0, 0, 0)
            .unwrap()
            .and_utc();

        let mut tx = self.pool.begin().await?;

        sqlx::query(
            r#"
            UPDATE app_activity_logs
            SET session_id = NULL
            WHERE user_id = $1 AND recorded_at >= $2 AND recorded_at < $3
            "#,
        )
        .bind(user_id)
        .bind(start)
        .bind(end)
        .execute(&mut *tx)
        .await?;

        sqlx::query(
            r#"
            UPDATE website_logs
            SET session_id = NULL
            WHERE user_id = $1 AND visited_at >= $2 AND visited_at < $3
            "#,
        )
        .bind(user_id)
        .bind(start)
        .bind(end)
        .execute(&mut *tx)
        .await?;

        sqlx::query(
            r#"
            DELETE FROM sessions
            WHERE user_id = $1 AND start_time >= $2 AND start_time < $3
            "#,
        )
        .bind(user_id)
        .bind(start)
        .bind(end)
        .execute(&mut *tx)
        .await?;

        tx.commit().await?;
        Ok(())
    }

    pub async fn link_app_logs(&self, session_id: Uuid, log_ids: &[Uuid]) -> anyhow::Result<()> {
        if log_ids.is_empty() {
            return Ok(());
        }

        sqlx::query(
            r#"
            UPDATE app_activity_logs
            SET session_id = $1
            WHERE id = ANY($2)
            "#,
        )
        .bind(session_id)
        .bind(log_ids)
        .execute(self.pool)
        .await?;

        Ok(())
    }

    pub async fn link_website_logs_in_window(
        &self,
        session_id: Uuid,
        user_id: Uuid,
        start: DateTime<Utc>,
        end: DateTime<Utc>,
    ) -> anyhow::Result<u64> {
        let result = sqlx::query(
            r#"
            UPDATE website_logs
            SET session_id = $1
            WHERE user_id = $2
              AND visited_at >= $3
              AND visited_at <= $4
              AND (session_id IS NULL OR session_id = $1)
            "#,
        )
        .bind(session_id)
        .bind(user_id)
        .bind(start)
        .bind(end)
        .execute(self.pool)
        .await?;

        Ok(result.rows_affected())
    }

    pub async fn list_website_logs_for_session(
        &self,
        session_id: Uuid,
    ) -> anyhow::Result<Vec<crate::models::WebsiteLogRow>> {
        let rows = sqlx::query_as::<_, crate::models::WebsiteLogRow>(
            r#"
            SELECT id, url, domain, time_spent_sec, category::text AS category, visited_at, session_id
            FROM website_logs
            WHERE session_id = $1
            ORDER BY visited_at ASC
            "#,
        )
        .bind(session_id)
        .fetch_all(self.pool)
        .await?;

        Ok(rows)
    }
}
