use axum::{extract::State, routing::get, Json, Router};
use netchronicle_db::ActivityRepository;
use serde::Serialize;

use crate::error::ApiResult;
use crate::params::DateRangeParams;
use crate::query::UserQuery;
use crate::state::AppState;

#[derive(Serialize)]
#[serde(rename_all = "camelCase")]
pub struct TimelineEntry {
    pub time: String,
    pub label: String,
    pub category: String,
    pub source: String,
    pub duration_sec: i32,
    #[serde(skip_serializing_if = "Option::is_none")]
    pub session_id: Option<String>,
}

#[derive(Serialize)]
#[serde(rename_all = "camelCase")]
pub struct TimelineResponse {
    pub date: String,
    pub entries: Vec<TimelineEntry>,
}

pub fn router() -> Router<AppState> {
    Router::new().route("/timeline", get(timeline))
}

async fn timeline(
    State(state): State<AppState>,
    user: UserQuery,
    range: DateRangeParams,
) -> ApiResult<Json<TimelineResponse>> {
    let activity = ActivityRepository::new(&state.db);

    let apps = activity
        .list_app_logs(user.user_id, range.from, range.to, range.limit, range.offset)
        .await
        .map_err(|e| crate::error::ApiError::internal(e.to_string()))?;
    let sites = activity
        .list_website_logs(user.user_id, range.from, range.to, range.limit, range.offset)
        .await
        .map_err(|e| crate::error::ApiError::internal(e.to_string()))?;

    let mut entries = Vec::new();

    for row in apps {
        let start = row.recorded_at - chrono::Duration::seconds(row.duration_sec as i64);
        entries.push(TimelineEntry {
            time: start.to_rfc3339(),
            label: row
                .window_title
                .unwrap_or_else(|| row.app_name.clone()),
            category: row.category,
            source: "app".into(),
            duration_sec: row.duration_sec,
            session_id: row.session_id.map(|id| id.to_string()),
        });
    }

    for row in sites {
        let start = row.visited_at - chrono::Duration::seconds(row.time_spent_sec as i64);
        entries.push(TimelineEntry {
            time: start.to_rfc3339(),
            label: row.domain,
            category: row.category,
            source: "website".into(),
            duration_sec: row.time_spent_sec,
            session_id: row.session_id.map(|id| id.to_string()),
        });
    }

    entries.sort_by(|a, b| a.time.cmp(&b.time));

    Ok(Json(TimelineResponse {
        date: range.from.date_naive().to_string(),
        entries,
    }))
}
