use axum::{
    extract::{Path, State},
    routing::{delete, get},
    Json, Router,
};
use netchronicle_common::ActivityCategory;
use netchronicle_db::{parse_category, CategoryRuleRepository};
use serde::{Deserialize, Serialize};
use uuid::Uuid;

use crate::error::{ApiError, ApiResult};
use crate::query::UserQuery;
use crate::state::AppState;

#[derive(Deserialize)]
#[serde(rename_all = "camelCase")]
pub struct CreateRuleRequest {
    pub pattern: String,
    pub pattern_type: String,
    pub category: String,
    #[serde(default)]
    pub priority: i32,
}

#[derive(Serialize)]
#[serde(rename_all = "camelCase")]
pub struct CategoryRulesResponse {
    pub rules: Vec<CategoryRuleItem>,
}

#[derive(Serialize)]
#[serde(rename_all = "camelCase")]
pub struct CategoryRuleItem {
    pub id: String,
    pub pattern: String,
    pub pattern_type: String,
    pub category: String,
    pub priority: i32,
}

pub fn router() -> Router<AppState> {
    Router::new()
        .route("/category-rules", get(list_rules).post(create_rule))
        .route("/category-rules/{id}", delete(delete_rule))
}

async fn list_rules(
    State(state): State<AppState>,
    user: UserQuery,
) -> ApiResult<Json<CategoryRulesResponse>> {
    let rows = CategoryRuleRepository::new(&state.db)
        .list(user.user_id)
        .await
        .map_err(|e| ApiError::internal(e.to_string()))?;

    let rules = rows
        .into_iter()
        .map(|row| CategoryRuleItem {
            id: row.id.to_string(),
            pattern: row.pattern,
            pattern_type: row.pattern_type,
            category: row.category,
            priority: row.priority,
        })
        .collect();

    Ok(Json(CategoryRulesResponse { rules }))
}

async fn create_rule(
    State(state): State<AppState>,
    user: UserQuery,
    Json(body): Json<CreateRuleRequest>,
) -> ApiResult<Json<CategoryRuleItem>> {
    if body.pattern.trim().is_empty() {
        return Err(ApiError::bad_request("pattern is required"));
    }

    let category = parse_category(&body.category);
    if !matches!(
        category,
        ActivityCategory::Work
            | ActivityCategory::Learning
            | ActivityCategory::Entertainment
            | ActivityCategory::Distraction
            | ActivityCategory::Neutral
            | ActivityCategory::Unknown
    ) {
        return Err(ApiError::bad_request("invalid category"));
    }

    let row = CategoryRuleRepository::new(&state.db)
        .create(
            user.user_id,
            body.pattern.trim(),
            body.pattern_type.trim(),
            category,
            body.priority,
        )
        .await
        .map_err(|e| ApiError::internal(e.to_string()))?;

    Ok(Json(CategoryRuleItem {
        id: row.id.to_string(),
        pattern: row.pattern,
        pattern_type: row.pattern_type,
        category: row.category,
        priority: row.priority,
    }))
}

async fn delete_rule(
    State(state): State<AppState>,
    user: UserQuery,
    Path(id): Path<Uuid>,
) -> ApiResult<Json<serde_json::Value>> {
    let deleted = CategoryRuleRepository::new(&state.db)
        .delete(user.user_id, id)
        .await
        .map_err(|e| ApiError::internal(e.to_string()))?;

    if !deleted {
        return Err(ApiError::not_found("rule not found"));
    }

    Ok(Json(serde_json::json!({ "deleted": true })))
}
