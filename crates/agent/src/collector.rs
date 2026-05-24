use tracing::info;

use netchronicle_categorization::{Categorizer, RuleStore};

use crate::config::AgentConfig;

pub async fn run(config: AgentConfig) -> anyhow::Result<()> {
    let categorizer = Categorizer::new(RuleStore::with_defaults());
    let _ = categorizer;

    info!(
        interval_secs = config.network_sample_interval.as_secs(),
        "collector initialized (platform hooks pending)"
    );

    // TODO: OS/browser hooks, persist raw_events, invoke session-builder pipeline
    tokio::signal::ctrl_c().await?;
    info!("agent shutting down");
    Ok(())
}
