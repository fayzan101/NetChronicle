use axum::{extract::State, routing::get, Json, Router};
use chrono::Utc;
use netchronicle_db::AnalyticsRepository;
use serde::Serialize;

use crate::query::{day_bounds, UserQuery};
use crate::state::AppState;

#[derive(Serialize)]
#[serde(rename_all = "camelCase")]
pub struct WeeklyReportResponse {
    pub week_start: String,
    pub week_end: String,
    pub summary: serde_json::Value,
}

pub fn router() -> Router<AppState> {
    Router::new().route("/weekly-report", get(weekly_report))
}

async fn weekly_report(
    State(state): State<AppState>,
    user: UserQuery,
) -> Result<Json<WeeklyReportResponse>, (axum::http::StatusCode, String)> {
    let today = Utc::now().date_naive();
    let week_start = today - chrono::Duration::days(6);
    let (from, _) = day_bounds(week_start);
    let (_, to) = day_bounds(today);

    let analytics = AnalyticsRepository::new(&state.db);
    let breakdown = analytics
        .category_breakdown(user.user_id, from, to)
        .await
        .map_err(internal_error)?;
    let top_apps = analytics
        .top_apps(user.user_id, from, to, 10)
        .await
        .map_err(internal_error)?;
    let top_domains = analytics
        .top_domains(user.user_id, from, to, 10)
        .await
        .map_err(internal_error)?;

    let total_sec: i64 = breakdown.iter().map(|row| row.total_sec).sum();
    let productive_sec: i64 = breakdown
        .iter()
        .filter(|row| row.category == "work" || row.category == "learning")
        .map(|row| row.total_sec)
        .sum();

    let summary = serde_json::json!({
        "total_online_minutes": total_sec / 60,
        "productive_minutes": productive_sec / 60,
        "category_breakdown": breakdown,
        "top_apps": top_apps.into_iter().map(|(name, secs)| serde_json::json!({"app": name, "minutes": secs / 60})).collect::<Vec<_>>(),
        "top_domains": top_domains.into_iter().map(|(domain, secs)| serde_json::json!({"domain": domain, "minutes": secs / 60})).collect::<Vec<_>>(),
    });

    Ok(Json(WeeklyReportResponse {
        week_start: week_start.to_string(),
        week_end: today.to_string(),
        summary,
    }))
}

fn internal_error(error: impl std::fmt::Display) -> (axum::http::StatusCode, String) {
    (
        axum::http::StatusCode::INTERNAL_SERVER_ERROR,
        error.to_string(),
    )
}
