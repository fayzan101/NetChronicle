//! NetChronicle background agent — collects app/site/network activity.

mod collector;
mod config;

use anyhow::Context;
use tracing::info;
use tracing_subscriber::EnvFilter;

use crate::config::AgentConfig;

#[tokio::main]
async fn main() -> anyhow::Result<()> {
    tracing_subscriber::fmt()
        .with_env_filter(EnvFilter::from_default_env())
        .init();

    let config = AgentConfig::from_env().context("invalid agent configuration")?;
    info!(user_id = %config.user_id, "starting NetChronicle agent");

    collector::run(config).await
}
