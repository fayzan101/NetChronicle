use axum::Json;
use axum::Router;
use serde::Serialize;
use tower_http::cors::{Any, CorsLayer};
use tower_http::trace::TraceLayer;

use netchronicle_db::DbPool;

use crate::routes;
use crate::state::AppState;

#[derive(Serialize)]
#[serde(rename_all = "camelCase")]
struct HealthResponse {
    status: &'static str,
    database: &'static str,
}

pub fn create_app(db: DbPool, auth_required: bool) -> Router {
    let state = AppState::new(db.clone(), auth_required);

    Router::new()
        .merge(routes::api_router())
        .route("/health", axum::routing::get(health_check))
        .route("/metrics", axum::routing::get(metrics))
        .layer(TraceLayer::new_for_http())
        .layer(
            CorsLayer::new()
                .allow_origin(Any)
                .allow_methods(Any)
                .allow_headers(Any),
        )
        .with_state(state)
}

async fn health_check(
    axum::extract::State(state): axum::extract::State<AppState>,
) -> Json<HealthResponse> {
    let database = if sqlx::query("SELECT 1").execute(&state.db).await.is_ok() {
        "ok"
    } else {
        "error"
    };

    Json(HealthResponse {
        status: "ok",
        database,
    })
}

async fn metrics(
    axum::extract::State(state): axum::extract::State<AppState>,
) -> Result<String, (axum::http::StatusCode, String)> {
    let sessions: i64 = sqlx::query_scalar("SELECT COUNT(*)::bigint FROM sessions")
        .fetch_one(&state.db)
        .await
        .map_err(|e| (axum::http::StatusCode::INTERNAL_SERVER_ERROR, e.to_string()))?;
    let network_logs: i64 = sqlx::query_scalar("SELECT COUNT(*)::bigint FROM network_logs")
        .fetch_one(&state.db)
        .await
        .map_err(|e| (axum::http::StatusCode::INTERNAL_SERVER_ERROR, e.to_string()))?;
    let reports: i64 = sqlx::query_scalar("SELECT COUNT(*)::bigint FROM reports")
        .fetch_one(&state.db)
        .await
        .map_err(|e| (axum::http::StatusCode::INTERNAL_SERVER_ERROR, e.to_string()))?;
    let raw_events: i64 = sqlx::query_scalar("SELECT COUNT(*)::bigint FROM raw_events")
        .fetch_one(&state.db)
        .await
        .map_err(|e| (axum::http::StatusCode::INTERNAL_SERVER_ERROR, e.to_string()))?;

    let body = format!(
        "# HELP netchronicle_sessions_total Total sessions stored\n\
         # TYPE netchronicle_sessions_total gauge\n\
         netchronicle_sessions_total {sessions}\n\
         # HELP netchronicle_network_logs_total Total network samples stored\n\
         # TYPE netchronicle_network_logs_total gauge\n\
         netchronicle_network_logs_total {network_logs}\n\
         # HELP netchronicle_reports_total Cached reports stored\n\
         # TYPE netchronicle_reports_total gauge\n\
         netchronicle_reports_total {reports}\n\
         # HELP netchronicle_raw_events_total Raw events stored\n\
         # TYPE netchronicle_raw_events_total gauge\n\
         netchronicle_raw_events_total {raw_events}\n"
    );

    Ok(body)
}
