use axum::{extract::State, routing::get, Json, Router};
use netchronicle_db::NetworkRepository;
use serde::Serialize;

use crate::error::ApiResult;
use crate::params::DateRangeParams;
use crate::query::UserQuery;
use crate::state::AppState;

#[derive(Serialize)]
#[serde(rename_all = "camelCase")]
pub struct NetworkStatsResponse {
    pub samples: Vec<NetworkSamplePoint>,
    pub aggregation: NetworkAggregationResponse,
    pub stability_score: f32,
}

#[derive(Serialize)]
#[serde(rename_all = "camelCase")]
pub struct NetworkAggregationResponse {
    pub sample_count: i64,
    pub avg_latency_ms: Option<f32>,
    pub p95_latency_ms: Option<f32>,
    pub avg_packet_loss_pct: Option<f32>,
    pub avg_bandwidth_mbps: Option<f32>,
    pub disconnect_count: i64,
}

#[derive(Serialize)]
#[serde(rename_all = "camelCase")]
pub struct NetworkSamplePoint {
    pub recorded_at: String,
    pub latency_ms: Option<f32>,
    pub packet_loss_pct: Option<f32>,
    pub bandwidth_mbps: Option<f32>,
    pub stability: Option<String>,
    pub disconnect: bool,
}

#[derive(Serialize)]
#[serde(rename_all = "camelCase")]
pub struct NetworkEventsResponse {
    pub events: Vec<NetworkEventPoint>,
}

#[derive(Serialize)]
#[serde(rename_all = "camelCase")]
pub struct NetworkEventPoint {
    pub recorded_at: String,
    pub kind: String,
    pub latency_ms: Option<f32>,
    pub packet_loss_pct: Option<f32>,
    pub bandwidth_mbps: Option<f32>,
    pub stability: Option<String>,
    pub disconnect: bool,
}

pub fn router() -> Router<AppState> {
    Router::new()
        .route("/network-stats", get(network_stats))
        .route("/network-events", get(network_events))
}

async fn network_stats(
    State(state): State<AppState>,
    user: UserQuery,
    range: DateRangeParams,
) -> ApiResult<Json<NetworkStatsResponse>> {
    let repo = NetworkRepository::new(&state.db);

    let rows = repo
        .list_range(user.user_id, range.from, range.to, range.limit)
        .await
        .map_err(|e| crate::error::ApiError::internal(e.to_string()))?;
    let aggregation = repo
        .aggregate(user.user_id, range.from, range.to)
        .await
        .map_err(|e| crate::error::ApiError::internal(e.to_string()))?;
    let stability_score = repo
        .stability_score(user.user_id, range.from)
        .await
        .map_err(|e| crate::error::ApiError::internal(e.to_string()))?;

    let samples = rows
        .into_iter()
        .map(|row| NetworkSamplePoint {
            recorded_at: row.recorded_at.to_rfc3339(),
            latency_ms: row.latency_ms,
            packet_loss_pct: row.packet_loss_pct,
            bandwidth_mbps: row.bandwidth_mbps,
            stability: row.stability,
            disconnect: row.disconnect,
        })
        .collect();

    Ok(Json(NetworkStatsResponse {
        samples,
        aggregation: NetworkAggregationResponse {
            sample_count: aggregation.sample_count,
            avg_latency_ms: aggregation.avg_latency_ms,
            p95_latency_ms: aggregation.p95_latency_ms,
            avg_packet_loss_pct: aggregation.avg_packet_loss_pct,
            avg_bandwidth_mbps: aggregation.avg_bandwidth_mbps,
            disconnect_count: aggregation.disconnect_count,
        },
        stability_score,
    }))
}

async fn network_events(
    State(state): State<AppState>,
    user: UserQuery,
    range: DateRangeParams,
) -> ApiResult<Json<NetworkEventsResponse>> {
    let events = NetworkRepository::new(&state.db)
        .list_events(user.user_id, range.from, range.to, range.limit)
        .await
        .map_err(|e| crate::error::ApiError::internal(e.to_string()))?
        .into_iter()
        .map(|row| NetworkEventPoint {
            recorded_at: row.recorded_at.to_rfc3339(),
            kind: row.kind,
            latency_ms: row.latency_ms,
            packet_loss_pct: row.packet_loss_pct,
            bandwidth_mbps: row.bandwidth_mbps,
            stability: row.stability,
            disconnect: row.disconnect,
        })
        .collect();

    Ok(Json(NetworkEventsResponse { events }))
}
