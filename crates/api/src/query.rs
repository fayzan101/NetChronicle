use axum::extract::FromRequestParts;
use axum::http::request::Parts;
use netchronicle_common::hash_token;
use netchronicle_db::{ApiKeyRepository, AuthTokenRepository, UserRepository};
use uuid::Uuid;

use crate::error::ApiError;
use crate::state::AppState;

#[derive(Debug, Clone, Copy)]
pub struct AuthUser {
    pub user_id: Uuid,
}

fn bearer_token(parts: &Parts) -> Option<String> {
    let header = parts
        .headers
        .get(axum::http::header::AUTHORIZATION)?
        .to_str()
        .ok()?;
    header
        .strip_prefix("Bearer ")
        .or_else(|| header.strip_prefix("bearer "))
        .map(|s| s.trim().to_string())
        .filter(|s| !s.is_empty())
}

fn api_key_header(parts: &Parts) -> Option<String> {
    parts
        .headers
        .get("x-api-key")
        .and_then(|v| v.to_str().ok())
        .map(|s| s.trim().to_string())
        .filter(|s| !s.is_empty())
}

async fn resolve_token(state: &AppState, token: &str) -> Result<Uuid, ApiError> {
    let token_hash = hash_token(token);

    if let Some(row) = AuthTokenRepository::new(&state.db)
        .find_valid(&token_hash)
        .await
        .map_err(|e| ApiError::internal(e.to_string()))?
    {
        return Ok(row.user_id);
    }

    if let Some(key) = ApiKeyRepository::new(&state.db)
        .find_by_hash(&token_hash)
        .await
        .map_err(|e| ApiError::internal(e.to_string()))?
    {
        let _ = ApiKeyRepository::new(&state.db).touch(key.id).await;
        return Ok(key.user_id);
    }

    Err(ApiError::unauthorized("invalid or expired token"))
}

impl FromRequestParts<AppState> for AuthUser {
    type Rejection = ApiError;

    async fn from_request_parts(
        parts: &mut Parts,
        state: &AppState,
    ) -> Result<Self, Self::Rejection> {
        if let Some(token) = bearer_token(parts).or_else(|| api_key_header(parts)) {
            let user_id = resolve_token(state, &token).await?;
            return Ok(Self { user_id });
        }

        if state.auth_required {
            return Err(ApiError::unauthorized(
                "missing Authorization bearer token or X-Api-Key",
            ));
        }

        // Local-dev fallback
        let query = parts.uri.query().unwrap_or_default();
        let requested = query.split('&').find_map(|pair| {
            let (key, value) = pair.split_once('=')?;
            (key == "user_id").then(|| value.to_string())
        });
        if let Some(id) = requested
            .and_then(|v| Uuid::parse_str(&v).ok())
            .filter(|id| !id.is_nil())
        {
            return Ok(Self { user_id: id });
        }

        if let Ok(value) = std::env::var("DEFAULT_USER_ID") {
            if let Ok(id) = Uuid::parse_str(&value) {
                if !id.is_nil() {
                    return Ok(Self { user_id: id });
                }
            }
        }

        UserRepository::new(&state.db)
            .ensure_local_user()
            .await
            .map(|user_id| Self { user_id })
            .map_err(|error| ApiError::internal(format!("failed to resolve user: {error}")))
    }
}

/// Back-compat alias used by existing routes.
pub type UserQuery = AuthUser;
