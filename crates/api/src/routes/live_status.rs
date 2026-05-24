use axum::{routing::get, Json, Router};
use serde::Serialize;

use crate::state::AppState;

#[derive(Serialize)]
pub struct LiveStatusResponse {
    pub current_app: Option<String>,
    pub current_site: Option<String>,
    pub focus_score: f32,
    pub session_elapsed_sec: u32,
    pub network_latency_ms: Option<f32>,
}

pub fn router() -> Router<AppState> {
    Router::new().route("/live-status", get(live_status))
}

async fn live_status() -> Json<LiveStatusResponse> {
    Json(LiveStatusResponse {
        current_app: None,
        current_site: None,
        focus_score: 0.0,
        session_elapsed_sec: 0,
        network_latency_ms: None,
    })
}
