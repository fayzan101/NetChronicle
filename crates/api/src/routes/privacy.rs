use axum::{
    extract::State,
    http::{header, StatusCode},
    response::{IntoResponse, Response},
    routing::{delete, post},
    Json, Router,
};
use netchronicle_common::hash_token;
use netchronicle_db::{
    session_row_to_common, ActivityRepository, NetworkRepository, SessionRepository, UserRepository,
};
use serde::Deserialize;
use serde_json::json;

use crate::error::{ApiError, ApiResult};
use crate::query::AuthUser;
use crate::state::AppState;

#[derive(Deserialize)]
#[serde(rename_all = "camelCase")]
pub struct ExportRequest {
    pub format: Option<String>,
}

#[derive(Deserialize)]
#[serde(rename_all = "camelCase")]
pub struct DeleteDataRequest {
    /// Must equal SHA-256 hex of `DELETE:{user_id}` (or the convenience token from GET).
    pub confirmation: String,
}

pub fn router() -> Router<AppState> {
    Router::new()
        .route("/export", post(export_data))
        .route("/data/delete-token", post(delete_token))
        .route("/data", delete(delete_data))
}

async fn export_data(
    State(state): State<AppState>,
    user: AuthUser,
    Json(body): Json<ExportRequest>,
) -> Result<Response, ApiError> {
    let format = body
        .format
        .as_deref()
        .unwrap_or("json")
        .to_ascii_lowercase();

    let activity = ActivityRepository::new(&state.db);
    let mut payload = activity
        .export_payload(user.user_id)
        .await
        .map_err(|e| ApiError::internal(e.to_string()))?;

    let sessions = SessionRepository::new(&state.db)
        .list(
            user.user_id,
            chrono::DateTime::from_timestamp(0, 0).unwrap_or_else(chrono::Utc::now),
            chrono::Utc::now() + chrono::Duration::days(1),
            10_000,
            0,
        )
        .await
        .map_err(|e| ApiError::internal(e.to_string()))?
        .into_iter()
        .map(session_row_to_common)
        .map(|s| {
            json!({
                "sessionId": s.session_id,
                "startTime": s.start_time,
                "endTime": s.end_time,
                "category": format!("{:?}", s.category).to_lowercase(),
                "productivityScore": s.productivity_score,
                "primaryApps": s.primary_apps,
            })
        })
        .collect::<Vec<_>>();
    payload["sessions"] = json!(sessions);

    let network = NetworkRepository::new(&state.db)
        .list_since(
            user.user_id,
            chrono::DateTime::from_timestamp(0, 0).unwrap_or_else(chrono::Utc::now),
            50_000,
        )
        .await
        .map_err(|e| ApiError::internal(e.to_string()))?;
    payload["networkLogs"] = json!(network
        .into_iter()
        .map(|row| json!({
            "latencyMs": row.latency_ms,
            "packetLossPct": row.packet_loss_pct,
            "bandwidthMbps": row.bandwidth_mbps,
            "stability": row.stability,
            "disconnect": row.disconnect,
            "recordedAt": row.recorded_at,
        }))
        .collect::<Vec<_>>());

    if format == "csv" {
        let csv = export_csv(&payload);
        return Ok((
            StatusCode::OK,
            [(header::CONTENT_TYPE, "text/csv; charset=utf-8")],
            csv,
        )
            .into_response());
    }

    Ok((
        StatusCode::OK,
        [(header::CONTENT_TYPE, "application/json")],
        payload.to_string(),
    )
        .into_response())
}

fn export_csv(payload: &serde_json::Value) -> String {
    let mut lines = vec!["type,key,value".to_string()];
    if let Some(apps) = payload["appActivityLogs"].as_array() {
        for app in apps {
            lines.push(format!(
                "app,{},{}",
                app["appName"].as_str().unwrap_or(""),
                app["durationSec"].as_i64().unwrap_or(0)
            ));
        }
    }
    if let Some(sites) = payload["websiteLogs"].as_array() {
        for site in sites {
            lines.push(format!(
                "website,{},{}",
                site["domain"].as_str().unwrap_or(""),
                site["timeSpentSec"].as_i64().unwrap_or(0)
            ));
        }
    }
    lines.join("\n") + "\n"
}

async fn delete_token(user: AuthUser) -> ApiResult<Json<serde_json::Value>> {
    let token = hash_token(&format!("DELETE:{}", user.user_id));
    Ok(Json(json!({
        "confirmation": token,
        "instruction": "Send this confirmation value in DELETE /data to wipe activity."
    })))
}

async fn delete_data(
    State(state): State<AppState>,
    user: AuthUser,
    Json(body): Json<DeleteDataRequest>,
) -> ApiResult<Json<serde_json::Value>> {
    let expected = hash_token(&format!("DELETE:{}", user.user_id));
    if body.confirmation.trim() != expected {
        return Err(ApiError::bad_request(
            "invalid confirmation token — call POST /data/delete-token first",
        ));
    }

    let deleted = UserRepository::new(&state.db)
        .wipe_activity_data(user.user_id)
        .await
        .map_err(|e| ApiError::internal(e.to_string()))?;

    Ok(Json(json!({
        "deletedRows": deleted,
        "userId": user.user_id,
    })))
}
