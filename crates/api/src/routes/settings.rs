use axum::{extract::State, routing::get, Json, Router};
use netchronicle_db::{UserRepository, UserSettings};
use serde::Deserialize;

use crate::error::{ApiError, ApiResult};
use crate::query::AuthUser;
use crate::state::AppState;

#[derive(Deserialize)]
#[serde(rename_all = "camelCase")]
pub struct PatchSettingsRequest {
    pub tracking_enabled: Option<bool>,
    pub poll_interval_secs: Option<u64>,
    pub idle_threshold_secs: Option<u64>,
    pub network_sample_interval_secs: Option<u64>,
    pub privacy_hide_titles: Option<bool>,
    pub privacy_hide_urls: Option<bool>,
}

pub fn router() -> Router<AppState> {
    Router::new().route("/settings", get(get_settings).patch(patch_settings))
}

async fn get_settings(
    State(state): State<AppState>,
    user: AuthUser,
) -> ApiResult<Json<UserSettings>> {
    let settings = UserRepository::new(&state.db)
        .get_settings(user.user_id)
        .await
        .map_err(|e| ApiError::internal(e.to_string()))?;
    Ok(Json(settings))
}

async fn patch_settings(
    State(state): State<AppState>,
    user: AuthUser,
    Json(body): Json<PatchSettingsRequest>,
) -> ApiResult<Json<UserSettings>> {
    let repo = UserRepository::new(&state.db);
    let mut settings = repo
        .get_settings(user.user_id)
        .await
        .map_err(|e| ApiError::internal(e.to_string()))?;

    if let Some(v) = body.tracking_enabled {
        settings.tracking_enabled = v;
    }
    if body.poll_interval_secs.is_some() {
        settings.poll_interval_secs = body.poll_interval_secs;
    }
    if body.idle_threshold_secs.is_some() {
        settings.idle_threshold_secs = body.idle_threshold_secs;
    }
    if body.network_sample_interval_secs.is_some() {
        settings.network_sample_interval_secs = body.network_sample_interval_secs;
    }
    if let Some(v) = body.privacy_hide_titles {
        settings.privacy_hide_titles = v;
    }
    if let Some(v) = body.privacy_hide_urls {
        settings.privacy_hide_urls = v;
    }

    let updated = repo
        .update_settings(user.user_id, &settings)
        .await
        .map_err(|e| ApiError::internal(e.to_string()))?;
    Ok(Json(updated))
}
