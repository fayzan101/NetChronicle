use axum::{
    extract::{Query, State},
    routing::get,
    Json, Router,
};
use chrono::Utc;
use netchronicle_db::{ActivityRepository, DeviceRepository, NetworkRepository};
use serde::{Deserialize, Serialize};
use uuid::Uuid;

use crate::error::{ApiError, ApiResult};
use crate::query::UserQuery;
use crate::state::AppState;

#[derive(Serialize)]
#[serde(rename_all = "camelCase")]
pub struct LiveStatusResponse {
    pub current_app: Option<String>,
    pub current_site: Option<String>,
    pub focus_score: f32,
    pub session_elapsed_sec: u32,
    pub network_latency_ms: Option<f32>,
    pub device_id: Option<String>,
    pub device_name: Option<String>,
}

#[derive(Deserialize)]
#[serde(rename_all = "camelCase")]
pub struct LiveQuery {
    pub device_id: Option<String>,
}

pub fn router() -> Router<AppState> {
    Router::new().route("/live-status", get(live_status))
}

async fn live_status(
    State(state): State<AppState>,
    user: UserQuery,
    Query(query): Query<LiveQuery>,
) -> ApiResult<Json<LiveStatusResponse>> {
    let devices = DeviceRepository::new(&state.db);
    let activity = ActivityRepository::new(&state.db);

    let device = if let Some(id) = query
        .device_id
        .as_deref()
        .and_then(|s| Uuid::parse_str(s).ok())
    {
        devices
            .get_by_id(user.user_id, id)
            .await
            .map_err(|e| ApiError::internal(e.to_string()))?
    } else {
        devices
            .latest_for_user(user.user_id)
            .await
            .map_err(|e| ApiError::internal(e.to_string()))?
    };

    let snapshot = if let Some(device) = &device {
        activity
            .latest_snapshot_for_device(user.user_id, device.id)
            .await
            .map_err(|e| ApiError::internal(e.to_string()))?
            .or(activity
                .latest_snapshot(user.user_id)
                .await
                .map_err(|e| ApiError::internal(e.to_string()))?)
    } else {
        activity
            .latest_snapshot(user.user_id)
            .await
            .map_err(|e| ApiError::internal(e.to_string()))?
    };

    let latency = NetworkRepository::new(&state.db)
        .latest_latency(user.user_id)
        .await
        .map_err(|e| ApiError::internal(e.to_string()))?;

    let mut response = LiveStatusResponse {
        current_app: None,
        current_site: None,
        focus_score: 0.0,
        session_elapsed_sec: 0,
        network_latency_ms: latency,
        device_id: device.as_ref().map(|d| d.id.to_string()),
        device_name: device.as_ref().map(|d| d.name.clone()),
    };

    if let Some(snapshot) = snapshot {
        let age = Utc::now().signed_duration_since(snapshot.recorded_at);
        if age.num_seconds() <= 60 {
            response.current_app = snapshot
                .payload
                .get("app")
                .and_then(|v| v.as_str())
                .map(str::to_string);
            response.current_site = snapshot
                .payload
                .get("domain")
                .and_then(|v| v.as_str())
                .map(str::to_string);
            response.session_elapsed_sec = snapshot
                .payload
                .get("durationSec")
                .or_else(|| snapshot.payload.get("duration_sec"))
                .and_then(|v| v.as_u64())
                .unwrap_or(0) as u32;
            response.focus_score = focus_score_from_category(
                snapshot.payload.get("category").and_then(|v| v.as_str()),
            );
        }
    }

    Ok(Json(response))
}

fn focus_score_from_category(category: Option<&str>) -> f32 {
    match category {
        Some("work") => 90.0,
        Some("learning") => 75.0,
        Some("neutral") => 50.0,
        Some("entertainment") => 35.0,
        Some("distraction") => 15.0,
        _ => 40.0,
    }
}
