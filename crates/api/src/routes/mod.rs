mod category_rules;
mod daily_report;
mod insights;
mod live_status;
mod network_stats;
mod sessions;
mod timeline;
mod weekly_report;

use axum::Router;

use crate::state::AppState;

pub fn api_router() -> Router<AppState> {
    Router::new()
        .merge(sessions::router())
        .merge(timeline::router())
        .merge(daily_report::router())
        .merge(weekly_report::router())
        .merge(live_status::router())
        .merge(network_stats::router())
        .merge(insights::router())
        .merge(category_rules::router())
}
