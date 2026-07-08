use axum::{extract::State, routing::get, Json, Router};
use netchronicle_analytics::AnalyticsEngine;
use netchronicle_db::{session_row_to_common, AnalyticsRepository, ReportRepository, SessionRepository};
use serde::Serialize;

use crate::error::ApiResult;
use crate::params::{day_bounds, DateRangeParams};
use crate::query::UserQuery;
use crate::state::AppState;

#[derive(Serialize)]
#[serde(rename_all = "camelCase")]
pub struct WeeklyReportResponse {
    pub week_start: String,
    pub week_end: String,
    pub summary: serde_json::Value,
    pub cached: bool,
}

pub fn router() -> Router<AppState> {
    Router::new().route("/weekly-report", get(weekly_report))
}

async fn weekly_report(
    State(state): State<AppState>,
    user: UserQuery,
    range: DateRangeParams,
) -> ApiResult<Json<WeeklyReportResponse>> {
    let week_end = range.to.date_naive() - chrono::Duration::days(1);
    let week_start = range.from.date_naive();
    let reports = ReportRepository::new(&state.db);

    if let Some(cached) = reports
        .get(user.user_id, "weekly", week_start, week_end)
        .await
        .map_err(|e| crate::error::ApiError::internal(e.to_string()))?
    {
        return Ok(Json(WeeklyReportResponse {
            week_start: week_start.to_string(),
            week_end: week_end.to_string(),
            summary: cached.summary,
            cached: true,
        }));
    }

    let (from, _to) = day_bounds(week_start);
    let (_, to_end) = day_bounds(week_end);
    let session_rows = SessionRepository::new(&state.db)
        .list(user.user_id, from, to_end, 5000, 0)
        .await
        .map_err(|e| crate::error::ApiError::internal(e.to_string()))?;
    let sessions: Vec<_> = session_rows.into_iter().map(session_row_to_common).collect();

    let weekly = AnalyticsEngine::weekly_summary(&sessions);
    let analytics = AnalyticsRepository::new(&state.db);
    let top_apps = analytics
        .top_apps(user.user_id, from, to_end, 10)
        .await
        .map_err(|e| crate::error::ApiError::internal(e.to_string()))?;
    let top_domains = analytics
        .top_domains(user.user_id, from, to_end, 10)
        .await
        .map_err(|e| crate::error::ApiError::internal(e.to_string()))?;

    let summary = serde_json::json!({
        "totalOnlineMinutes": weekly.total_online_minutes,
        "productiveMinutes": weekly.productive_minutes,
        "sessionCount": weekly.session_count,
        "averageProductivityScore": weekly.average_productivity_score,
        "categoryMinutes": weekly.category_minutes,
        "topApps": top_apps.into_iter().map(|(name, secs)| serde_json::json!({"app": name, "minutes": secs / 60})).collect::<Vec<_>>(),
        "topDomains": top_domains.into_iter().map(|(domain, secs)| serde_json::json!({"domain": domain, "minutes": secs / 60})).collect::<Vec<_>>(),
    });

    reports
        .upsert(user.user_id, "weekly", week_start, week_end, summary.clone())
        .await
        .map_err(|e| crate::error::ApiError::internal(e.to_string()))?;

    Ok(Json(WeeklyReportResponse {
        week_start: week_start.to_string(),
        week_end: week_end.to_string(),
        summary,
        cached: false,
    }))
}
