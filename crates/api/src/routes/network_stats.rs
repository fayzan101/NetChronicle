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
    pub stability_score: f32,
}

#[derive(Serialize)]
#[serde(rename_all = "camelCase")]
pub struct NetworkSamplePoint {
    pub recorded_at: String,
    pub latency_ms: Option<f32>,
    pub packet_loss_pct: Option<f32>,
}

pub fn router() -> Router<AppState> {
    Router::new().route("/network-stats", get(network_stats))
}

async fn network_stats(
    State(state): State<AppState>,
    user: UserQuery,
    range: DateRangeParams,
) -> ApiResult<Json<NetworkStatsResponse>> {
    let repo = NetworkRepository::new(&state.db);

    let rows = repo
        .list_since(user.user_id, range.from, range.limit)
        .await
        .map_err(|e| crate::error::ApiError::internal(e.to_string()))?;
    let stability_score = repo
        .stability_score(user.user_id, range.from)
        .await
        .map_err(|e| crate::error::ApiError::internal(e.to_string()))?;

    let samples = rows
        .into_iter()
        .rev()
        .filter(|row| row.recorded_at < range.to)
        .map(|row| NetworkSamplePoint {
            recorded_at: row.recorded_at.to_rfc3339(),
            latency_ms: row.latency_ms,
            packet_loss_pct: row.packet_loss_pct,
        })
        .collect();

    Ok(Json(NetworkStatsResponse {
        samples,
        stability_score,
    }))
}
