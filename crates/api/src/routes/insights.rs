use axum::{routing::get, Json, Router};
use serde::Serialize;

use crate::state::AppState;

#[derive(Serialize)]
pub struct InsightsResponse {
    pub insights: Vec<InsightItem>,
}

#[derive(Serialize)]
pub struct InsightItem {
    pub title: String,
    pub body: String,
    pub severity: String,
}

pub fn router() -> Router<AppState> {
    Router::new().route("/insights", get(insights))
}

async fn insights() -> Json<InsightsResponse> {
    Json(InsightsResponse {
        insights: vec![],
    })
}
