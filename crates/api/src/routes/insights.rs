use axum::{extract::State, routing::get, Json, Router};
use chrono::Utc;
use netchronicle_db::AnalyticsRepository;
use serde::Serialize;

use crate::query::{day_bounds, UserQuery};
use crate::state::AppState;

#[derive(Serialize)]
pub struct InsightsResponse {
    pub insights: Vec<InsightItem>,
}

#[derive(Serialize)]
pub struct InsightItem {
    pub title: String,
    pub body: String,
    pub severity: String,
}

pub fn router() -> Router<AppState> {
    Router::new().route("/insights", get(insights))
}

async fn insights(
    State(state): State<AppState>,
    user: UserQuery,
) -> Result<Json<InsightsResponse>, (axum::http::StatusCode, String)> {
    let today = Utc::now().date_naive();
    let (from, to) = day_bounds(today);
    let analytics = AnalyticsRepository::new(&state.db);

    let stats = analytics
        .daily_activity_stats(user.user_id, today)
        .await
        .map_err(internal_error)?;
    let top_apps = analytics
        .top_apps(user.user_id, from, to, 3)
        .await
        .map_err(internal_error)?;
    let top_domains = analytics
        .top_domains(user.user_id, from, to, 3)
        .await
        .map_err(internal_error)?;

    let mut insights = Vec::new();

    if stats.total_sec == 0 {
        insights.push(InsightItem {
            title: "Start tracking".into(),
            body: "Run the NetChronicle agent to begin collecting activity data.".into(),
            severity: "info".into(),
        });
    } else {
        let distraction_pct = (stats.distraction_sec as f32 / stats.total_sec as f32) * 100.0;
        if distraction_pct > 20.0 {
            insights.push(InsightItem {
                title: "High distraction time".into(),
                body: format!(
                    "Distraction sites accounted for {:.0}% of tracked time today.",
                    distraction_pct
                ),
                severity: "warning".into(),
            });
        }

        let productive_pct = (stats.productive_sec as f32 / stats.total_sec as f32) * 100.0;
        if productive_pct >= 60.0 {
            insights.push(InsightItem {
                title: "Strong focus day".into(),
                body: format!(
                    "{:.0}% of your tracked time was work or learning.",
                    productive_pct
                ),
                severity: "positive".into(),
            });
        }

        if let Some((app, secs)) = top_apps.first() {
            insights.push(InsightItem {
                title: "Most used app".into(),
                body: format!("You spent {} minutes in {} today.", secs / 60, app),
                severity: "info".into(),
            });
        }

        if let Some((domain, secs)) = top_domains.first() {
            insights.push(InsightItem {
                title: "Top website".into(),
                body: format!("{} was your most visited site ({} minutes).", domain, secs / 60),
                severity: "info".into(),
            });
        }
    }

    Ok(Json(InsightsResponse { insights }))
}

fn internal_error(error: impl std::fmt::Display) -> (axum::http::StatusCode, String) {
    (
        axum::http::StatusCode::INTERNAL_SERVER_ERROR,
        error.to_string(),
    )
}
