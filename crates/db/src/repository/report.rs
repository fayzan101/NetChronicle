use chrono::NaiveDate;
use sqlx::PgPool;
use uuid::Uuid;

use crate::models::ReportRow;

pub struct ReportRepository<'a> {
    pool: &'a PgPool,
}

impl<'a> ReportRepository<'a> {
    pub fn new(pool: &'a PgPool) -> Self {
        Self { pool }
    }

    pub async fn get(
        &self,
        user_id: Uuid,
        report_type: &str,
        period_start: NaiveDate,
        period_end: NaiveDate,
    ) -> anyhow::Result<Option<ReportRow>> {
        let row = sqlx::query_as::<_, ReportRow>(
            r#"
            SELECT id, report_type, period_start, period_end, summary, created_at
            FROM reports
            WHERE user_id = $1 AND report_type = $2 AND period_start = $3 AND period_end = $4
            "#,
        )
        .bind(user_id)
        .bind(report_type)
        .bind(period_start)
        .bind(period_end)
        .fetch_optional(self.pool)
        .await?;

        Ok(row)
    }

    pub async fn upsert(
        &self,
        user_id: Uuid,
        report_type: &str,
        period_start: NaiveDate,
        period_end: NaiveDate,
        summary: serde_json::Value,
    ) -> anyhow::Result<ReportRow> {
        let row = sqlx::query_as::<_, ReportRow>(
            r#"
            INSERT INTO reports (user_id, report_type, period_start, period_end, summary)
            VALUES ($1, $2, $3, $4, $5)
            ON CONFLICT (user_id, report_type, period_start, period_end)
            DO UPDATE SET summary = EXCLUDED.summary, created_at = now()
            RETURNING id, report_type, period_start, period_end, summary, created_at
            "#,
        )
        .bind(user_id)
        .bind(report_type)
        .bind(period_start)
        .bind(period_end)
        .bind(summary)
        .fetch_one(self.pool)
        .await?;

        Ok(row)
    }

    pub async fn list(
        &self,
        user_id: Uuid,
        report_type: Option<&str>,
        limit: i64,
    ) -> anyhow::Result<Vec<ReportRow>> {
        let rows = if let Some(rtype) = report_type {
            sqlx::query_as::<_, ReportRow>(
                r#"
                SELECT id, report_type, period_start, period_end, summary, created_at
                FROM reports
                WHERE user_id = $1 AND report_type = $2
                ORDER BY period_start DESC
                LIMIT $3
                "#,
            )
            .bind(user_id)
            .bind(rtype)
            .bind(limit)
            .fetch_all(self.pool)
            .await?
        } else {
            sqlx::query_as::<_, ReportRow>(
                r#"
                SELECT id, report_type, period_start, period_end, summary, created_at
                FROM reports
                WHERE user_id = $1
                ORDER BY period_start DESC
                LIMIT $2
                "#,
            )
            .bind(user_id)
            .bind(limit)
            .fetch_all(self.pool)
            .await?
        };

        Ok(rows)
    }
}
