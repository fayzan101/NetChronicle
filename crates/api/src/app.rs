use axum::Router;
use tower_http::cors::{Any, CorsLayer};
use tower_http::trace::TraceLayer;

use netchronicle_db::DbPool;

use crate::routes;
use crate::state::AppState;

pub fn create_app(db: DbPool) -> Router {
    let state = AppState::new(db);

    Router::new()
        .merge(routes::api_router())
        .route("/health", axum::routing::get(|| async { "ok" }))
        .layer(TraceLayer::new_for_http())
        .layer(
            CorsLayer::new()
                .allow_origin(Any)
                .allow_methods(Any)
                .allow_headers(Any),
        )
        .with_state(state)
}
