use axum::{extract::State, routing::get, Json, Router};
use netchronicle_db::{session_row_to_common, SessionRepository};
use serde::Serialize;

use crate::error::ApiResult;
use crate::params::DateRangeParams;
use crate::query::UserQuery;
use crate::state::AppState;

#[derive(Serialize)]
#[serde(rename_all = "camelCase")]
pub struct SessionItem {
    pub session_id: String,
    pub start_time: String,
    pub end_time: Option<String>,
    pub category: String,
    pub productivity_score: Option<f32>,
    pub primary_apps: Vec<String>,
    pub network_stability: Option<String>,
}

#[derive(Serialize)]
#[serde(rename_all = "camelCase")]
pub struct SessionsResponse {
    pub sessions: Vec<SessionItem>,
    pub limit: i64,
    pub offset: i64,
}

pub fn router() -> Router<AppState> {
    Router::new().route("/sessions", get(list_sessions))
}

async fn list_sessions(
    State(state): State<AppState>,
    user: UserQuery,
    range: DateRangeParams,
) -> ApiResult<Json<SessionsResponse>> {
    let rows = SessionRepository::new(&state.db)
        .list(user.user_id, range.from, range.to, range.limit, range.offset)
        .await
        .map_err(|error| crate::error::ApiError::internal(error.to_string()))?;

    let sessions = rows
        .into_iter()
        .map(|row| {
            let session = session_row_to_common(row);
            SessionItem {
                session_id: session.session_id.to_string(),
                start_time: session.start_time.to_rfc3339(),
                end_time: session.end_time.map(|t| t.to_rfc3339()),
                category: format!("{:?}", session.category).to_lowercase(),
                productivity_score: session.productivity_score,
                primary_apps: session.primary_apps,
                network_stability: session
                    .network_stability
                    .map(|s| format!("{:?}", s).to_lowercase()),
            }
        })
        .collect();

    Ok(Json(SessionsResponse {
        sessions,
        limit: range.limit,
        offset: range.offset,
    }))
}
