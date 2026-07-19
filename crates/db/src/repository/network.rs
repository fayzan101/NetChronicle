use chrono::{DateTime, Utc};
use netchronicle_common::NetworkStability;
use sqlx::PgPool;
use uuid::Uuid;

use crate::models::{stability_to_db, NetworkLogRow};

#[derive(Debug, Clone)]
pub struct NetworkAggregation {
    pub sample_count: i64,
    pub avg_latency_ms: Option<f32>,
    pub p95_latency_ms: Option<f32>,
    pub avg_packet_loss_pct: Option<f32>,
    pub avg_bandwidth_mbps: Option<f32>,
    pub disconnect_count: i64,
}

#[derive(Debug, Clone)]
pub struct NetworkEventRow {
    pub recorded_at: DateTime<Utc>,
    pub kind: String,
    pub latency_ms: Option<f32>,
    pub packet_loss_pct: Option<f32>,
    pub bandwidth_mbps: Option<f32>,
    pub stability: Option<String>,
    pub disconnect: bool,
}

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

    pub async fn list_range(
        &self,
        user_id: Uuid,
        from: DateTime<Utc>,
        to: DateTime<Utc>,
        limit: i64,
    ) -> anyhow::Result<Vec<NetworkLogRow>> {
        let rows = sqlx::query_as::<_, NetworkLogRow>(
            r#"
            SELECT latency_ms, packet_loss_pct, bandwidth_mbps, stability::text AS stability, disconnect, recorded_at
            FROM network_logs
            WHERE user_id = $1 AND recorded_at >= $2 AND recorded_at < $3
            ORDER BY recorded_at ASC
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

    pub async fn aggregate(
        &self,
        user_id: Uuid,
        from: DateTime<Utc>,
        to: DateTime<Utc>,
    ) -> anyhow::Result<NetworkAggregation> {
        let rows = self.list_range(user_id, from, to, 10_000).await?;
        Ok(aggregate_samples(&rows))
    }

    pub async fn list_events(
        &self,
        user_id: Uuid,
        from: DateTime<Utc>,
        to: DateTime<Utc>,
        limit: i64,
    ) -> anyhow::Result<Vec<NetworkEventRow>> {
        let rows = self.list_range(user_id, from, to, 10_000).await?;
        let mut events: Vec<NetworkEventRow> = rows
            .into_iter()
            .filter(|row| {
                row.disconnect
                    || netchronicle_network_monitor::is_spike(row.latency_ms, row.packet_loss_pct)
            })
            .map(|row| {
                let kind = if row.disconnect {
                    "disconnect".to_string()
                } else {
                    "spike".to_string()
                };
                NetworkEventRow {
                    recorded_at: row.recorded_at,
                    kind,
                    latency_ms: row.latency_ms,
                    packet_loss_pct: row.packet_loss_pct,
                    bandwidth_mbps: row.bandwidth_mbps,
                    stability: row.stability,
                    disconnect: row.disconnect,
                }
            })
            .collect();

        // Newest first for event feeds.
        events.sort_by(|a, b| b.recorded_at.cmp(&a.recorded_at));
        events.truncate(limit.max(1) as usize);
        Ok(events)
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

pub fn aggregate_samples(rows: &[NetworkLogRow]) -> NetworkAggregation {
    let sample_count = rows.len() as i64;
    let disconnect_count = rows.iter().filter(|r| r.disconnect).count() as i64;

    let latencies: Vec<f32> = rows.iter().filter_map(|r| r.latency_ms).collect();
    let losses: Vec<f32> = rows.iter().filter_map(|r| r.packet_loss_pct).collect();
    let bandwidths: Vec<f32> = rows.iter().filter_map(|r| r.bandwidth_mbps).collect();

    let avg_latency_ms = mean(&latencies);
    let p95_latency_ms = percentile(&latencies, 0.95);
    let avg_packet_loss_pct = mean(&losses);
    let avg_bandwidth_mbps = mean(&bandwidths);

    NetworkAggregation {
        sample_count,
        avg_latency_ms,
        p95_latency_ms,
        avg_packet_loss_pct,
        avg_bandwidth_mbps,
        disconnect_count,
    }
}

fn mean(values: &[f32]) -> Option<f32> {
    if values.is_empty() {
        None
    } else {
        Some(values.iter().sum::<f32>() / values.len() as f32)
    }
}

fn percentile(values: &[f32], p: f32) -> Option<f32> {
    if values.is_empty() {
        return None;
    }
    let mut sorted = values.to_vec();
    sorted.sort_by(|a, b| a.partial_cmp(b).unwrap_or(std::cmp::Ordering::Equal));
    let idx = ((sorted.len() as f32 - 1.0) * p).round() as usize;
    sorted.get(idx.clamp(0, sorted.len() - 1)).copied()
}

#[cfg(test)]
mod tests {
    use super::*;
    use chrono::TimeZone;

    #[test]
    fn aggregates_latency_and_loss() {
        let rows = vec![
            NetworkLogRow {
                latency_ms: Some(10.0),
                packet_loss_pct: Some(0.0),
                bandwidth_mbps: Some(50.0),
                stability: Some("stable".into()),
                disconnect: false,
                recorded_at: Utc.with_ymd_and_hms(2026, 1, 1, 9, 0, 0).unwrap(),
            },
            NetworkLogRow {
                latency_ms: Some(20.0),
                packet_loss_pct: Some(10.0),
                bandwidth_mbps: Some(40.0),
                stability: Some("degraded".into()),
                disconnect: false,
                recorded_at: Utc.with_ymd_and_hms(2026, 1, 1, 9, 1, 0).unwrap(),
            },
            NetworkLogRow {
                latency_ms: None,
                packet_loss_pct: Some(100.0),
                bandwidth_mbps: None,
                stability: Some("offline".into()),
                disconnect: true,
                recorded_at: Utc.with_ymd_and_hms(2026, 1, 1, 9, 2, 0).unwrap(),
            },
        ];

        let agg = aggregate_samples(&rows);
        assert_eq!(agg.sample_count, 3);
        assert_eq!(agg.disconnect_count, 1);
        assert_eq!(agg.avg_latency_ms, Some(15.0));
        assert_eq!(agg.avg_packet_loss_pct, Some(110.0 / 3.0));
        assert_eq!(agg.avg_bandwidth_mbps, Some(45.0));
        assert_eq!(agg.p95_latency_ms, Some(20.0));
    }
}
