use std::sync::Arc;
use std::time::{Duration, Instant};

use chrono::Utc;
use tokio::sync::RwLock;

#[derive(Debug, Clone)]
pub struct BrowserTabReport {
    pub url: String,
    pub title: Option<String>,
    pub received_at: Instant,
}

impl BrowserTabReport {
    pub fn is_fresh(&self, max_age: Duration) -> bool {
        self.received_at.elapsed() <= max_age
    }
}

#[derive(Clone, Default)]
pub struct BrowserFeed(Arc<RwLock<Option<BrowserTabReport>>>);

impl BrowserFeed {
    pub fn new() -> Self {
        Self::default()
    }

    pub async fn update(&self, url: String, title: Option<String>) {
        let mut guard = self.0.write().await;
        *guard = Some(BrowserTabReport {
            url,
            title,
            received_at: Instant::now(),
        });
    }

    pub async fn latest_fresh(&self, max_age: Duration) -> Option<BrowserTabReport> {
        let guard = self.0.read().await;
        guard.as_ref().filter(|tab| tab.is_fresh(max_age)).cloned()
    }
}

#[derive(serde::Deserialize)]
pub struct BrowserTabPayload {
    pub url: String,
    pub title: Option<String>,
    pub tab_id: Option<i64>,
    pub active: Option<bool>,
}

pub async fn run_browser_feed_server(feed: BrowserFeed, port: u16) -> anyhow::Result<()> {
    use axum::{routing::post, Router};

    let app = Router::new()
        .route("/browser-tab", post(handle_browser_tab))
        .with_state(feed);

    let addr = format!("127.0.0.1:{port}");
    let listener = tokio::net::TcpListener::bind(&addr).await?;
    tracing::info!(%addr, "browser feed server listening");
    axum::serve(listener, app).await?;
    Ok(())
}

async fn handle_browser_tab(
    axum::extract::State(feed): axum::extract::State<BrowserFeed>,
    axum::Json(payload): axum::Json<BrowserTabPayload>,
) -> &'static str {
    if payload.active == Some(false) {
        return "ok";
    }

    if !payload.url.trim().is_empty() {
        tracing::debug!(
            url = %payload.url,
            tab_id = ?payload.tab_id,
            at = %Utc::now(),
            "browser tab reported"
        );
        feed.update(payload.url, payload.title).await;
    }
    "ok"
}
