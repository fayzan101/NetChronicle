use axum::{routing::get, Json, Router};
use serde::Serialize;

use crate::state::AppState;

#[derive(Serialize)]
pub struct NetworkStatsResponse {
    pub samples: Vec<NetworkSamplePoint>,
    pub stability_score: f32,
}

#[derive(Serialize)]
pub struct NetworkSamplePoint {
    pub recorded_at: String,
    pub latency_ms: Option<f32>,
    pub packet_loss_pct: Option<f32>,
}

pub fn router() -> Router<AppState> {
    Router::new().route("/network-stats", get(network_stats))
}

async fn network_stats() -> Json<NetworkStatsResponse> {
    Json(NetworkStatsResponse {
        samples: vec![],
        stability_score: 0.0,
    })
}
