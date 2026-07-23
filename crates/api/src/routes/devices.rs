use axum::{
    extract::State,
    routing::{get, post},
    Json, Router,
};
use netchronicle_db::DeviceRepository;
use serde::{Deserialize, Serialize};

use crate::error::{ApiError, ApiResult};
use crate::query::AuthUser;
use crate::state::AppState;

#[derive(Serialize)]
#[serde(rename_all = "camelCase")]
pub struct DeviceItem {
    pub id: String,
    pub agent_id: String,
    pub name: String,
    pub last_seen: String,
    pub created_at: String,
}

#[derive(Deserialize)]
#[serde(rename_all = "camelCase")]
pub struct RegisterDeviceRequest {
    pub agent_id: String,
    pub name: Option<String>,
}

#[derive(Deserialize)]
#[serde(rename_all = "camelCase")]
pub struct HeartbeatRequest {
    pub agent_id: String,
}

pub fn router() -> Router<AppState> {
    Router::new()
        .route("/devices", get(list_devices).post(register_device))
        .route("/devices/heartbeat", post(heartbeat))
}

async fn list_devices(
    State(state): State<AppState>,
    user: AuthUser,
) -> ApiResult<Json<Vec<DeviceItem>>> {
    let rows = DeviceRepository::new(&state.db)
        .list_for_user(user.user_id)
        .await
        .map_err(|e| ApiError::internal(e.to_string()))?;

    Ok(Json(
        rows.into_iter()
            .map(|row| DeviceItem {
                id: row.id.to_string(),
                agent_id: row.agent_id,
                name: row.name,
                last_seen: row.last_seen.to_rfc3339(),
                created_at: row.created_at.to_rfc3339(),
            })
            .collect(),
    ))
}

async fn register_device(
    State(state): State<AppState>,
    user: AuthUser,
    Json(body): Json<RegisterDeviceRequest>,
) -> ApiResult<Json<DeviceItem>> {
    let agent_id = body.agent_id.trim();
    if agent_id.is_empty() {
        return Err(ApiError::bad_request("agentId required"));
    }
    let name = body
        .name
        .as_deref()
        .map(str::trim)
        .filter(|s| !s.is_empty())
        .unwrap_or("Agent");

    let row = DeviceRepository::new(&state.db)
        .upsert(user.user_id, agent_id, name)
        .await
        .map_err(|e| ApiError::internal(e.to_string()))?;

    Ok(Json(DeviceItem {
        id: row.id.to_string(),
        agent_id: row.agent_id,
        name: row.name,
        last_seen: row.last_seen.to_rfc3339(),
        created_at: row.created_at.to_rfc3339(),
    }))
}

async fn heartbeat(
    State(state): State<AppState>,
    user: AuthUser,
    Json(body): Json<HeartbeatRequest>,
) -> ApiResult<Json<serde_json::Value>> {
    let agent_id = body.agent_id.trim();
    if agent_id.is_empty() {
        return Err(ApiError::bad_request("agentId required"));
    }
    DeviceRepository::new(&state.db)
        .touch(user.user_id, agent_id)
        .await
        .map_err(|e| ApiError::internal(e.to_string()))?;
    Ok(Json(serde_json::json!({ "ok": true })))
}
