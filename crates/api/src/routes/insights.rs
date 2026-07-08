use axum::{extract::State, routing::get, Json, Router};
use netchronicle_analytics::AnalyticsEngine;
use netchronicle_db::{session_row_to_common, AnalyticsRepository, SessionRepository};
use serde::Serialize;

use crate::error::ApiResult;
use crate::params::DateRangeParams;
use crate::query::UserQuery;
use crate::state::AppState;

#[derive(Serialize)]
#[serde(rename_all = "camelCase")]
pub struct InsightsResponse {
    pub insights: Vec<InsightItem>,
}

#[derive(Serialize)]
#[serde(rename_all = "camelCase")]
pub struct InsightItem {
    pub title: String,
    pub body: String,
    pub severity: String,
}

pub fn router() -> Router<AppState> {
    Router::new().route("/insights", get(insights))
}

async fn insights(
    State(state): State<AppState>,
    user: UserQuery,
    range: DateRangeParams,
) -> ApiResult<Json<InsightsResponse>> {
    let session_rows = SessionRepository::new(&state.db)
        .list(user.user_id, range.from, range.to, 1000, 0)
        .await
        .map_err(|e| crate::error::ApiError::internal(e.to_string()))?;
    let sessions: Vec<_> = session_rows.into_iter().map(session_row_to_common).collect();

    let analytics = AnalyticsRepository::new(&state.db);
    let top_apps = analytics
        .top_apps(user.user_id, range.from, range.to, 3)
        .await
        .map_err(|e| crate::error::ApiError::internal(e.to_string()))?;
    let top_domains = analytics
        .top_domains(user.user_id, range.from, range.to, 3)
        .await
        .map_err(|e| crate::error::ApiError::internal(e.to_string()))?;

    let generated = AnalyticsEngine::generate_insights(&sessions, &top_apps, &top_domains);
    let insights = generated
        .into_iter()
        .map(|item| InsightItem {
            title: item.title,
            body: item.body,
            severity: format!("{:?}", item.severity).to_lowercase(),
        })
        .collect();

    Ok(Json(InsightsResponse { insights }))
}
