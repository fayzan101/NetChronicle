use std::env;

use axum::extract::FromRequestParts;
use axum::http::request::Parts;
use netchronicle_db::{DbPool, UserRepository};
use uuid::Uuid;

use crate::state::AppState;

#[derive(Debug, Clone, Copy)]
pub struct UserQuery {
    pub user_id: Uuid,
}

impl FromRequestParts<AppState> for UserQuery {
    type Rejection = (axum::http::StatusCode, String);

    async fn from_request_parts(
        parts: &mut Parts,
        state: &AppState,
    ) -> Result<Self, Self::Rejection> {
        let query = parts.uri.query().unwrap_or_default();
        let requested = query
            .split('&')
            .find_map(|pair| {
                let (key, value) = pair.split_once('=')?;
                (key == "user_id").then(|| value.to_string())
            })
            .and_then(|value| Uuid::parse_str(&value).ok());

        if let Some(id) = requested.filter(|id| !id.is_nil()) {
            return Ok(Self { user_id: id });
        }

        if let Ok(value) = env::var("DEFAULT_USER_ID") {
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
            .map_err(|error| {
                (
                    axum::http::StatusCode::INTERNAL_SERVER_ERROR,
                    format!("failed to resolve user: {error}"),
                )
            })
    }
}

pub fn day_bounds(day: chrono::NaiveDate) -> (chrono::DateTime<chrono::Utc>, chrono::DateTime<chrono::Utc>) {
    let start = day.and_hms_opt(0, 0, 0).unwrap().and_utc();
    let end = (day + chrono::Duration::days(1))
        .and_hms_opt(0, 0, 0)
        .unwrap()
        .and_utc();
    (start, end)
}

#[allow(dead_code)]
pub fn pool_ref(state: &AppState) -> &DbPool {
    &state.db
}
