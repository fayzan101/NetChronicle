use axum::{routing::get, Json, Router};
use serde::Serialize;

use crate::state::AppState;

#[derive(Serialize)]
pub struct SessionsResponse {
    pub sessions: Vec<serde_json::Value>,
}

pub fn router() -> Router<AppState> {
    Router::new().route("/sessions", get(list_sessions))
}

async fn list_sessions() -> Json<SessionsResponse> {
    Json(SessionsResponse {
        sessions: vec![],
    })
}
