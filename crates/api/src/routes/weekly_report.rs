use axum::{routing::get, Json, Router};
use serde::Serialize;

use crate::state::AppState;

#[derive(Serialize)]
pub struct WeeklyReportResponse {
    pub week_start: String,
    pub week_end: String,
    pub summary: serde_json::Value,
}

pub fn router() -> Router<AppState> {
    Router::new().route("/weekly-report", get(weekly_report))
}

async fn weekly_report() -> Json<WeeklyReportResponse> {
    let today = chrono::Utc::now().date_naive();
    Json(WeeklyReportResponse {
        week_start: (today - chrono::Duration::days(6)).to_string(),
        week_end: today.to_string(),
        summary: serde_json::json!({}),
    })
}
