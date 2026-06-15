use anyhow::Context;use netchronicle_categorization::{Categorizer, RuleStore};
use netchronicle_db::{create_pool, run_migrations, UserRepository};
use tracing::{info, warn};

use crate::config::AgentConfig;
use crate::tracker::{run_network_sampler, ActivityTracker};
use crate::window::current_foreground;

pub async fn run(config: AgentConfig) -> anyhow::Result<()> {
    let pool = create_pool(&config.database_url)
        .await
        .context("connect to database")?;
    run_migrations(&pool).await.context("run migrations")?;

    let user_id = UserRepository::new(&pool)
        .get_or_create(config.user_id)
        .await
        .context("resolve user")?;

    info!(%user_id, "tracking for user");

    let categorizer = Categorizer::new(RuleStore::with_defaults());
    let poll_secs = config.poll_interval.as_secs().max(1) as u32;
    let mut tracker = ActivityTracker::new(
        user_id,
        pool.clone(),
        categorizer,
        config.min_segment_secs,
        poll_secs,
    );

    let network_pool = pool.clone();
    let network_interval = config.network_sample_interval;
    tokio::spawn(async move {
        run_network_sampler(user_id, network_pool, network_interval).await;
    });

    let mut interval = tokio::time::interval(config.poll_interval);
    info!(
        poll_secs = config.poll_interval.as_secs(),
        network_secs = config.network_sample_interval.as_secs(),
        "agent running — press Ctrl+C to stop"
    );

    loop {
        tokio::select! {
            _ = interval.tick() => {
                match current_foreground() {
                    Ok(window) => {
                        if let Err(error) = tracker.tick(window).await {
                            warn!(%error, "failed to process foreground window");
                        }
                    }
                    Err(error) => warn!(%error, "failed to read foreground window"),
                }
            }
            result = tokio::signal::ctrl_c() => {
                result.context("ctrl-c handler")?;
                break;
            }
        }
    }

    tracker.shutdown().await.context("flush final segment")?;
    info!("agent shutting down");
    Ok(())
}
