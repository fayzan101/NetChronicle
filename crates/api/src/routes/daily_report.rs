use axum::{extract::State, routing::get, Json, Router};
use netchronicle_analytics::{AnalyticsEngine, DailyAnalyticsInput};
use netchronicle_db::{
    session_row_to_common, NetworkRepository, ReportRepository, SessionRepository,
};
use serde::Serialize;

use crate::error::ApiResult;
use crate::params::{day_bounds, DateRangeParams};
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
    pub cached: bool,
}

pub fn router() -> Router<AppState> {
    Router::new().route("/daily-report", get(daily_report))
}

async fn daily_report(
    State(state): State<AppState>,
    user: UserQuery,
    range: DateRangeParams,
) -> ApiResult<Json<DailyReportResponse>> {
    let day = range.from.date_naive();
    let reports = ReportRepository::new(&state.db);

    if let Some(cached) = reports
        .get(user.user_id, "daily", day, day)
        .await
        .map_err(|e| crate::error::ApiError::internal(e.to_string()))?
    {
        let summary = cached.summary;
        return Ok(Json(DailyReportResponse {
            date: day.to_string(),
            productivity_score: summary["productivityScore"].as_f64().unwrap_or(0.0) as f32,
            total_online_minutes: summary["totalOnlineMinutes"].as_u64().unwrap_or(0) as u32,
            network_health_score: summary["networkHealthScore"].as_f64().unwrap_or(0.0) as f32,
            distraction_ratio: summary["distractionRatio"].as_f64().unwrap_or(0.0) as f32,
            focus_minutes: summary["focusMinutes"].as_u64().unwrap_or(0) as u32,
            cached: true,
        }));
    }

    let (from, to) = day_bounds(day);
    let session_rows = SessionRepository::new(&state.db)
        .list(user.user_id, from, to, 1000, 0)
        .await
        .map_err(|e| crate::error::ApiError::internal(e.to_string()))?;
    let sessions: Vec<_> = session_rows
        .into_iter()
        .map(session_row_to_common)
        .collect();

    let network_score = NetworkRepository::new(&state.db)
        .stability_score(user.user_id, from)
        .await
        .map_err(|e| crate::error::ApiError::internal(e.to_string()))?;

    let summary = AnalyticsEngine::daily_summary(&DailyAnalyticsInput {
        date: day,
        sessions: sessions.clone(),
        network_health_score: network_score,
    });

    let payload = serde_json::json!({
        "productivityScore": summary.productivity_score,
        "totalOnlineMinutes": summary.total_online_minutes,
        "networkHealthScore": summary.network_health_score,
        "distractionRatio": summary.distraction_ratio,
        "distractionImpactPct": AnalyticsEngine::distraction_impact_pct(&sessions),
        "focusMinutes": summary.focus_minutes,
        "timeOfDay": AnalyticsEngine::time_of_day_patterns(&sessions),
    });
    reports
        .upsert(user.user_id, "daily", day, day, payload)
        .await
        .map_err(|e| crate::error::ApiError::internal(e.to_string()))?;

    Ok(Json(DailyReportResponse {
        date: day.to_string(),
        productivity_score: summary.productivity_score,
        total_online_minutes: summary.total_online_minutes,
        network_health_score: summary.network_health_score,
        distraction_ratio: summary.distraction_ratio,
        focus_minutes: summary.focus_minutes,
        cached: false,
    }))
}
