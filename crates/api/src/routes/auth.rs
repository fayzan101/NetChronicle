use axum::{
    extract::{Path, State},
    routing::{delete, get, post},
    Json, Router,
};
use chrono::{Duration, Utc};
use netchronicle_common::{
    api_key_prefix, generate_api_key, generate_bearer_token, hash_secret, hash_token, verify_secret,
};
use netchronicle_db::{ApiKeyRepository, AuthTokenRepository, UserRepository};
use serde::{Deserialize, Serialize};
use uuid::Uuid;

use crate::error::{ApiError, ApiResult};
use crate::query::AuthUser;
use crate::state::AppState;

#[derive(Deserialize)]
#[serde(rename_all = "camelCase")]
pub struct RegisterRequest {
    pub email: String,
    pub password: String,
    pub display_name: Option<String>,
}

#[derive(Deserialize)]
#[serde(rename_all = "camelCase")]
pub struct LoginRequest {
    pub email: String,
    pub password: String,
}

#[derive(Serialize)]
#[serde(rename_all = "camelCase")]
pub struct AuthResponse {
    pub user_id: String,
    pub email: Option<String>,
    pub display_name: String,
    pub token: String,
    pub expires_at: String,
}

#[derive(Deserialize)]
#[serde(rename_all = "camelCase")]
pub struct CreateApiKeyRequest {
    pub name: Option<String>,
}

#[derive(Serialize)]
#[serde(rename_all = "camelCase")]
pub struct CreateApiKeyResponse {
    pub id: String,
    pub name: String,
    pub key_prefix: String,
    pub api_key: String,
    pub created_at: String,
}

#[derive(Serialize)]
#[serde(rename_all = "camelCase")]
pub struct ApiKeyItem {
    pub id: String,
    pub name: String,
    pub key_prefix: String,
    pub created_at: String,
    pub last_used_at: Option<String>,
    pub revoked: bool,
}

pub fn router() -> Router<AppState> {
    Router::new()
        .route("/auth/register", post(register))
        .route("/auth/login", post(login))
        .route("/auth/api-keys", get(list_api_keys).post(create_api_key))
        .route("/auth/api-keys/{id}", delete(revoke_api_key))
}

async fn register(
    State(state): State<AppState>,
    Json(body): Json<RegisterRequest>,
) -> ApiResult<Json<AuthResponse>> {
    let email = body.email.trim();
    if email.is_empty() || body.password.len() < 8 {
        return Err(ApiError::bad_request(
            "email required and password must be at least 8 characters",
        ));
    }

    let users = UserRepository::new(&state.db);
    if users
        .get_by_email(email)
        .await
        .map_err(|e| ApiError::internal(e.to_string()))?
        .is_some()
    {
        return Err(ApiError::conflict("email already registered"));
    }

    let display_name = body
        .display_name
        .as_deref()
        .map(str::trim)
        .filter(|s| !s.is_empty())
        .unwrap_or(email);
    let password_hash = hash_secret(&body.password);
    let user = users
        .create_user(email, display_name, &password_hash)
        .await
        .map_err(|e| ApiError::internal(e.to_string()))?;

    issue_token(&state, user.id, user.email, user.display_name).await
}

async fn login(
    State(state): State<AppState>,
    Json(body): Json<LoginRequest>,
) -> ApiResult<Json<AuthResponse>> {
    let users = UserRepository::new(&state.db);
    let user = users
        .get_by_email(body.email.trim())
        .await
        .map_err(|e| ApiError::internal(e.to_string()))?
        .ok_or_else(|| ApiError::unauthorized("invalid email or password"))?;

    let Some(password_hash) = user.password_hash.as_deref() else {
        return Err(ApiError::unauthorized(
            "password login not configured for user",
        ));
    };
    if !verify_secret(&body.password, password_hash) {
        return Err(ApiError::unauthorized("invalid email or password"));
    }

    issue_token(&state, user.id, user.email, user.display_name).await
}

async fn issue_token(
    state: &AppState,
    user_id: Uuid,
    email: Option<String>,
    display_name: String,
) -> ApiResult<Json<AuthResponse>> {
    let token = generate_bearer_token();
    let expires_at = Utc::now() + Duration::days(30);
    AuthTokenRepository::new(&state.db)
        .insert(user_id, &hash_token(&token), expires_at)
        .await
        .map_err(|e| ApiError::internal(e.to_string()))?;

    Ok(Json(AuthResponse {
        user_id: user_id.to_string(),
        email,
        display_name,
        token,
        expires_at: expires_at.to_rfc3339(),
    }))
}

async fn create_api_key(
    State(state): State<AppState>,
    user: AuthUser,
    Json(body): Json<CreateApiKeyRequest>,
) -> ApiResult<Json<CreateApiKeyResponse>> {
    let name = body
        .name
        .as_deref()
        .map(str::trim)
        .filter(|s| !s.is_empty())
        .unwrap_or("agent");
    let api_key = generate_api_key();
    let row = ApiKeyRepository::new(&state.db)
        .insert(
            user.user_id,
            name,
            &api_key_prefix(&api_key),
            &hash_token(&api_key),
        )
        .await
        .map_err(|e| ApiError::internal(e.to_string()))?;

    Ok(Json(CreateApiKeyResponse {
        id: row.id.to_string(),
        name: row.name,
        key_prefix: row.key_prefix,
        api_key,
        created_at: row.created_at.to_rfc3339(),
    }))
}

async fn list_api_keys(
    State(state): State<AppState>,
    user: AuthUser,
) -> ApiResult<Json<Vec<ApiKeyItem>>> {
    let rows = ApiKeyRepository::new(&state.db)
        .list_for_user(user.user_id)
        .await
        .map_err(|e| ApiError::internal(e.to_string()))?;

    Ok(Json(
        rows.into_iter()
            .map(|row| ApiKeyItem {
                id: row.id.to_string(),
                name: row.name,
                key_prefix: row.key_prefix,
                created_at: row.created_at.to_rfc3339(),
                last_used_at: row.last_used_at.map(|t| t.to_rfc3339()),
                revoked: row.revoked_at.is_some(),
            })
            .collect(),
    ))
}

async fn revoke_api_key(
    State(state): State<AppState>,
    user: AuthUser,
    Path(id): Path<Uuid>,
) -> ApiResult<Json<serde_json::Value>> {
    let revoked = ApiKeyRepository::new(&state.db)
        .revoke(user.user_id, id)
        .await
        .map_err(|e| ApiError::internal(e.to_string()))?;
    if !revoked {
        return Err(ApiError::not_found("api key not found"));
    }
    Ok(Json(serde_json::json!({ "revoked": true })))
}
