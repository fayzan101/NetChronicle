use axum::{
    extract::{Path, State},
    routing::{get, put},
    Json, Router,
};
use netchronicle_common::ActivityCategory;
use netchronicle_db::{parse_category, ActivityRepository, CategoryRuleRepository};
use serde::{Deserialize, Serialize};
use uuid::Uuid;

use crate::error::{ApiError, ApiResult};
use crate::query::UserQuery;
use crate::state::AppState;

#[derive(Deserialize)]
#[serde(rename_all = "camelCase")]
pub struct RuleRequest {
    pub pattern: String,
    pub pattern_type: String,
    pub category: String,
    #[serde(default)]
    pub priority: i32,
}

#[derive(Deserialize)]
#[serde(rename_all = "camelCase")]
pub struct BrowserTabRequest {
    pub url: String,
    pub title: Option<String>,
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
        .route("/category-rules/{id}", put(update_rule).delete(delete_rule))
        .route("/browser-tab", axum::routing::post(report_browser_tab))
}

async fn list_rules(
    State(state): State<AppState>,
    user: UserQuery,
) -> ApiResult<Json<CategoryRulesResponse>> {
    let rows = CategoryRuleRepository::new(&state.db)
        .list(user.user_id)
        .await
        .map_err(|e| ApiError::internal(e.to_string()))?;

    let rules = rows.into_iter().map(row_to_item).collect();
    Ok(Json(CategoryRulesResponse { rules }))
}

async fn create_rule(
    State(state): State<AppState>,
    user: UserQuery,
    Json(body): Json<RuleRequest>,
) -> ApiResult<Json<CategoryRuleItem>> {
    let (pattern, pattern_type, category, priority) = validate_rule_request(&body)?;

    let row = CategoryRuleRepository::new(&state.db)
        .create(user.user_id, &pattern, &pattern_type, category, priority)
        .await
        .map_err(|e| ApiError::internal(e.to_string()))?;

    Ok(Json(row_to_item(row)))
}

async fn update_rule(
    State(state): State<AppState>,
    user: UserQuery,
    Path(id): Path<Uuid>,
    Json(body): Json<RuleRequest>,
) -> ApiResult<Json<CategoryRuleItem>> {
    let (pattern, pattern_type, category, priority) = validate_rule_request(&body)?;

    let row = CategoryRuleRepository::new(&state.db)
        .update(user.user_id, id, &pattern, &pattern_type, category, priority)
        .await
        .map_err(|e| ApiError::internal(e.to_string()))?
        .ok_or_else(|| ApiError::not_found("rule not found"))?;

    Ok(Json(row_to_item(row)))
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

async fn report_browser_tab(
    State(state): State<AppState>,
    user: UserQuery,
    Json(body): Json<BrowserTabRequest>,
) -> ApiResult<Json<serde_json::Value>> {
    if body.url.trim().is_empty() {
        return Err(ApiError::bad_request("url is required"));
    }

    ActivityRepository::new(&state.db)
        .insert_raw_event(
            user.user_id,
            "browser_tab",
            serde_json::json!({
                "url": body.url,
                "title": body.title,
            }),
            chrono::Utc::now(),
        )
        .await
        .map_err(|e| ApiError::internal(e.to_string()))?;

    Ok(Json(serde_json::json!({ "accepted": true })))
}

fn validate_rule_request(
    body: &RuleRequest,
) -> ApiResult<(String, String, ActivityCategory, i32)> {
    if body.pattern.trim().is_empty() {
        return Err(ApiError::bad_request("pattern is required"));
    }

    let category = parse_category(&body.category);
    Ok((
        body.pattern.trim().to_string(),
        body.pattern_type.trim().to_string(),
        category,
        body.priority,
    ))
}

fn row_to_item(row: netchronicle_db::CategoryRuleRow) -> CategoryRuleItem {
    CategoryRuleItem {
        id: row.id.to_string(),
        pattern: row.pattern,
        pattern_type: row.pattern_type,
        category: row.category,
        priority: row.priority,
    }
}
