//! NetChronicle background agent — collects app/site/network activity.

mod browser;
mod browser_feed;
mod collector;
mod config;
mod db_retry;
mod idle;
mod ignore;
mod rules_cache;
mod session_job;
mod settings_cache;
mod tracker;
mod window;

use anyhow::Context;
use tracing::info;
use tracing_subscriber::EnvFilter;

use crate::config::AgentConfig;

#[tokio::main]
async fn main() -> anyhow::Result<()> {
    dotenvy::dotenv().ok();

    tracing_subscriber::fmt()
        .with_env_filter(EnvFilter::from_default_env())
        .init();

    let config = AgentConfig::from_env().context("invalid agent configuration")?;
    info!(user_id = %config.user_id, "starting NetChronicle agent");

    collector::run(config).await
}
