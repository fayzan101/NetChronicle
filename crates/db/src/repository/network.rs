use chrono::{DateTime, Utc};
use netchronicle_common::NetworkStability;
use sqlx::PgPool;
use uuid::Uuid;

use crate::models::{stability_to_db, NetworkLogRow};

pub struct NetworkRepository<'a> {
    pool: &'a PgPool,
}

impl<'a> NetworkRepository<'a> {
    pub fn new(pool: &'a PgPool) -> Self {
        Self { pool }
    }

    pub async fn insert_log(
        &self,
        user_id: Uuid,
        latency_ms: Option<f32>,
        packet_loss_pct: Option<f32>,
        bandwidth_mbps: Option<f32>,
        stability: NetworkStability,
        disconnect: bool,
        recorded_at: DateTime<Utc>,
    ) -> anyhow::Result<()> {
        sqlx::query(
            r#"
            INSERT INTO network_logs
                (user_id, latency_ms, packet_loss_pct, bandwidth_mbps, stability, disconnect, recorded_at)
            VALUES ($1, $2, $3, $4, $5::network_stability, $6, $7)
            "#,
        )
        .bind(user_id)
        .bind(latency_ms)
        .bind(packet_loss_pct)
        .bind(bandwidth_mbps)
        .bind(stability_to_db(stability))
        .bind(disconnect)
        .bind(recorded_at)
        .execute(self.pool)
        .await?;

        Ok(())
    }

    pub async fn list_since(
        &self,
        user_id: Uuid,
        since: DateTime<Utc>,
        limit: i64,
    ) -> anyhow::Result<Vec<NetworkLogRow>> {
        let rows = sqlx::query_as::<_, NetworkLogRow>(
            r#"
            SELECT latency_ms, packet_loss_pct, bandwidth_mbps, stability::text AS stability, disconnect, recorded_at
            FROM network_logs
            WHERE user_id = $1 AND recorded_at >= $2
            ORDER BY recorded_at DESC
            LIMIT $3
            "#,
        )
        .bind(user_id)
        .bind(since)
        .bind(limit)
        .fetch_all(self.pool)
        .await?;

        Ok(rows)
    }

    pub async fn latest_latency(&self, user_id: Uuid) -> anyhow::Result<Option<f32>> {
        let value = sqlx::query_scalar::<_, Option<f32>>(
            r#"
            SELECT latency_ms
            FROM network_logs
            WHERE user_id = $1 AND disconnect = false
            ORDER BY recorded_at DESC
            LIMIT 1
            "#,
        )
        .bind(user_id)
        .fetch_optional(self.pool)
        .await?
        .flatten();

        Ok(value)
    }

    pub async fn stability_score(&self, user_id: Uuid, since: DateTime<Utc>) -> anyhow::Result<f32> {
        let row = sqlx::query_as::<_, (i64, i64)>(
            r#"
            SELECT
                COUNT(*) FILTER (WHERE disconnect = false AND stability IN ('stable', 'degraded'))::bigint,
                COUNT(*)::bigint
            FROM network_logs
            WHERE user_id = $1 AND recorded_at >= $2
            "#,
        )
        .bind(user_id)
        .bind(since)
        .fetch_one(self.pool)
        .await?;

        if row.1 == 0 {
            return Ok(100.0);
        }

        Ok((row.0 as f32 / row.1 as f32) * 100.0)
    }
}
