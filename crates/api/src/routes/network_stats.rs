use axum::{extract::State, routing::get, Json, Router};
use chrono::{Duration, Utc};
use netchronicle_db::NetworkRepository;
use serde::Serialize;

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
) -> Result<Json<NetworkStatsResponse>, (axum::http::StatusCode, String)> {
    let since = Utc::now() - Duration::hours(24);
    let repo = NetworkRepository::new(&state.db);

    let rows = repo
        .list_since(user.user_id, since, 500)
        .await
        .map_err(internal_error)?;
    let stability_score = repo
        .stability_score(user.user_id, since)
        .await
        .map_err(internal_error)?;

    let samples = rows
        .into_iter()
        .rev()
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

fn internal_error(error: impl std::fmt::Display) -> (axum::http::StatusCode, String) {
    (
        axum::http::StatusCode::INTERNAL_SERVER_ERROR,
        error.to_string(),
    )
}
