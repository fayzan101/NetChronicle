use axum::Router;
use axum::Json;
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

pub fn create_app(db: DbPool) -> Router {
    let state = AppState::new(db.clone());

    Router::new()
        .merge(routes::api_router())
        .route("/health", axum::routing::get(health_check))
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
    let database = if sqlx::query("SELECT 1")
        .execute(&state.db)
        .await
        .is_ok()
    {
        "ok"
    } else {
        "error"
    };

    Json(HealthResponse {
        status: "ok",
        database,
    })
}
