use anyhow::Context;
use netchronicle_db::{create_pool, run_migrations, UserRepository};
use tracing::{info, warn};

use crate::browser_feed::{run_browser_feed_server, BrowserFeed};
use crate::config::AgentConfig;
use crate::idle::is_user_idle;
use crate::ignore::should_ignore;
use crate::rules_cache::RulesCache;
use crate::session_job::run_session_rebuild_loop;
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

    let rules_cache = RulesCache::load(user_id, &pool).await?;
    rules_cache
        .clone()
        .spawn_refresh(user_id, pool.clone(), config.rules_refresh_interval);

    let browser_feed = BrowserFeed::new();
    let feed_for_server = browser_feed.clone();
    let browser_port = config.browser_feed_port;
    tokio::spawn(async move {
        if let Err(error) = run_browser_feed_server(feed_for_server, browser_port).await {
            warn!(%error, "browser feed server stopped");
        }
    });

    let poll_secs = config.poll_interval.as_secs().max(1) as u32;
    let ignore_apps = config.ignore_apps.clone();
    let idle_threshold = config.idle_threshold;
    let mut tracker = ActivityTracker::new(
        user_id,
        pool.clone(),
        rules_cache,
        browser_feed,
        config.min_segment_secs,
        poll_secs,
    );

    let network_pool = pool.clone();
    let network_interval = config.network_sample_interval;
    tokio::spawn(async move {
        run_network_sampler(user_id, network_pool, network_interval).await;
    });

    let session_pool = pool.clone();
    let session_interval = config.session_rebuild_interval;
    tokio::spawn(async move {
        run_session_rebuild_loop(user_id, session_pool, session_interval).await;
    });

    let mut interval = tokio::time::interval(config.poll_interval);
    info!(
        poll_secs = config.poll_interval.as_secs(),
        network_secs = config.network_sample_interval.as_secs(),
        session_rebuild_secs = config.session_rebuild_interval.as_secs(),
        rules_refresh_secs = config.rules_refresh_interval.as_secs(),
        idle_threshold_secs = config.idle_threshold.as_secs(),
        browser_feed_port = config.browser_feed_port,
        "agent running — press Ctrl+C to stop"
    );

    let mut user_idle = false;

    loop {
        tokio::select! {
            _ = interval.tick() => {
                let idle = is_user_idle(idle_threshold);
                if idle {
                    if !user_idle {
                        tracker.pause_current().await?;
                        user_idle = true;
                    }
                    continue;
                }
                user_idle = false;

                match current_foreground() {
                    Ok(window) => {
                        if should_ignore(&window.app_name, &window.window_title, &ignore_apps) {
                            continue;
                        }
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
