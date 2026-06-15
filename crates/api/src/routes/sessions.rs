use axum::{extract::State, routing::get, Json, Router};
use chrono::Utc;
use netchronicle_db::{parse_category, ActivityRepository};
use serde::Serialize;

use crate::query::{day_bounds, UserQuery};
use crate::state::AppState;

#[derive(Serialize)]
pub struct SessionItem {
    pub session_id: String,
    pub start_time: String,
    pub end_time: String,
    pub category: String,
    pub productivity_score: Option<f32>,
    pub primary_apps: Vec<String>,
    pub window_title: Option<String>,
    pub duration_sec: i32,
}

#[derive(Serialize)]
pub struct SessionsResponse {
    pub sessions: Vec<SessionItem>,
}

pub fn router() -> Router<AppState> {
    Router::new().route("/sessions", get(list_sessions))
}

async fn list_sessions(
    State(state): State<AppState>,
    user: UserQuery,
) -> Result<Json<SessionsResponse>, (axum::http::StatusCode, String)> {
    let today = Utc::now().date_naive();
    let (from, to) = day_bounds(today);

    let rows = ActivityRepository::new(&state.db)
        .list_app_logs(user.user_id, from, to, 200)
        .await
        .map_err(internal_error)?;

    let sessions = rows
        .into_iter()
        .map(|row| {
            let end = row.recorded_at;
            let start = end - chrono::Duration::seconds(row.duration_sec as i64);
            SessionItem {
                session_id: row.id.to_string(),
                start_time: start.to_rfc3339(),
                end_time: end.to_rfc3339(),
                category: row.category.clone(),
                productivity_score: category_score(&row.category),
                primary_apps: vec![row.app_name.clone()],
                window_title: row.window_title,
                duration_sec: row.duration_sec,
            }
        })
        .collect();

    Ok(Json(SessionsResponse { sessions }))
}

fn category_score(category: &str) -> Option<f32> {
    match parse_category(category) {
        netchronicle_common::ActivityCategory::Work => Some(90.0),
        netchronicle_common::ActivityCategory::Learning => Some(75.0),
        netchronicle_common::ActivityCategory::Neutral => Some(50.0),
        netchronicle_common::ActivityCategory::Entertainment => Some(35.0),
        netchronicle_common::ActivityCategory::Distraction => Some(15.0),
        netchronicle_common::ActivityCategory::Unknown => Some(40.0),
    }
}

fn internal_error(error: impl std::fmt::Display) -> (axum::http::StatusCode, String) {
    (
        axum::http::StatusCode::INTERNAL_SERVER_ERROR,
        error.to_string(),
    )
}
