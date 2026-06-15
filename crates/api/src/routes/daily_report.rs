use axum::{extract::State, routing::get, Json, Router};
use chrono::Utc;
use netchronicle_db::{AnalyticsRepository, NetworkRepository};
use serde::Serialize;

use crate::query::UserQuery;
use crate::state::AppState;

#[derive(Serialize)]
#[serde(rename_all = "camelCase")]
pub struct DailyReportResponse {
    pub date: String,
    pub productivity_score: f32,
    pub total_online_minutes: u32,
    pub network_health_score: f32,
    pub distraction_ratio: f32,
    pub focus_minutes: u32,
}

pub fn router() -> Router<AppState> {
    Router::new().route("/daily-report", get(daily_report))
}

async fn daily_report(
    State(state): State<AppState>,
    user: UserQuery,
) -> Result<Json<DailyReportResponse>, (axum::http::StatusCode, String)> {
    let today = Utc::now().date_naive();
    let analytics = AnalyticsRepository::new(&state.db);
    let stats = analytics
        .daily_activity_stats(user.user_id, today)
        .await
        .map_err(internal_error)?;

    let day_start = today.and_hms_opt(0, 0, 0).unwrap().and_utc();
    let network_score = NetworkRepository::new(&state.db)
        .stability_score(user.user_id, day_start)
        .await
        .map_err(internal_error)?;

    let total_online_minutes = (stats.total_sec / 60) as u32;
    let focus_minutes = (stats.productive_sec / 60) as u32;
    let distraction_ratio = if stats.total_sec > 0 {
        stats.distraction_sec as f32 / stats.total_sec as f32
    } else {
        0.0
    };
    let productivity_score = if stats.total_sec > 0 {
        (stats.productive_sec as f32 / stats.total_sec as f32) * 100.0
    } else {
        0.0
    };

    Ok(Json(DailyReportResponse {
        date: today.to_string(),
        productivity_score,
        total_online_minutes,
        network_health_score: network_score,
        distraction_ratio,
        focus_minutes,
    }))
}

fn internal_error(error: impl std::fmt::Display) -> (axum::http::StatusCode, String) {
    (
        axum::http::StatusCode::INTERNAL_SERVER_ERROR,
        error.to_string(),
    )
}
