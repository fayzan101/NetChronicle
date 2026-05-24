use axum::{routing::get, Json, Router};
use serde::Serialize;

use crate::state::AppState;

#[derive(Serialize)]
pub struct DailyReportResponse {
    pub date: String,
    pub productivity_score: f32,
    pub total_online_minutes: u32,
    pub network_health_score: f32,
    pub distraction_ratio: f32,
}

pub fn router() -> Router<AppState> {
    Router::new().route("/daily-report", get(daily_report))
}

async fn daily_report() -> Json<DailyReportResponse> {
    Json(DailyReportResponse {
        date: chrono::Utc::now().date_naive().to_string(),
        productivity_score: 0.0,
        total_online_minutes: 0,
        network_health_score: 0.0,
        distraction_ratio: 0.0,
    })
}
