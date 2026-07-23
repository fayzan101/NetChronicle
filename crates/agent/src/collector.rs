use anyhow::Context;
use netchronicle_common::hash_token;
use netchronicle_db::{
    create_pool, run_migrations, ApiKeyRepository, DeviceRepository, UserRepository,
};
use tracing::{info, warn};
use uuid::Uuid;

use crate::browser_feed::{run_browser_feed_server, BrowserFeed};
use crate::config::AgentConfig;
use crate::idle::is_user_idle;
use crate::ignore::should_ignore;
use crate::rules_cache::RulesCache;
use crate::session_job::run_session_rebuild_loop;
use crate::settings_cache::SettingsCache;
use crate::tracker::{run_network_sampler, ActivityTracker};
use crate::window::current_foreground;

pub async fn run(config: AgentConfig) -> anyhow::Result<()> {
    let pool = create_pool(&config.database_url)
        .await
        .context("connect to database")?;
    run_migrations(&pool).await.context("run migrations")?;

    let user_id = resolve_user_id(&pool, &config).await?;
    info!(%user_id, agent_id = %config.agent_id, "tracking for user");

    let device = DeviceRepository::new(&pool)
        .upsert(user_id, &config.agent_id, &config.device_name)
        .await
        .context("register device")?;
    info!(device_id = %device.id, name = %device.name, "device registered");

    let settings = UserRepository::new(&pool)
        .get_settings(user_id)
        .await
        .unwrap_or_default();
    let settings_cache = SettingsCache::new(settings);
    settings_cache
        .clone()
        .spawn_refresh(user_id, pool.clone(), config.settings_refresh_interval);

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
    let mut idle_threshold = config.idle_threshold;
    let mut tracker = ActivityTracker::new(
        user_id,
        pool.clone(),
        rules_cache,
        browser_feed,
        config.min_segment_secs,
        poll_secs,
        Some(device.id),
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

    let device_pool = pool.clone();
    let agent_id = config.agent_id.clone();
    tokio::spawn(async move {
        let mut ticker = tokio::time::interval(std::time::Duration::from_secs(60));
        loop {
            ticker.tick().await;
            if let Err(error) = DeviceRepository::new(&device_pool)
                .touch(user_id, &agent_id)
                .await
            {
                warn!(%error, "failed to touch device last_seen");
            }
        }
    });

    let mut interval = tokio::time::interval(config.poll_interval);
    info!(
        poll_secs = config.poll_interval.as_secs(),
        network_secs = config.network_sample_interval.as_secs(),
        session_rebuild_secs = config.session_rebuild_interval.as_secs(),
        rules_refresh_secs = config.rules_refresh_interval.as_secs(),
        settings_refresh_secs = config.settings_refresh_interval.as_secs(),
        idle_threshold_secs = config.idle_threshold.as_secs(),
        browser_feed_port = config.browser_feed_port,
        "agent running — press Ctrl+C to stop"
    );

    let mut user_idle = false;

    loop {
        tokio::select! {
            _ = interval.tick() => {
                let settings = settings_cache.get().await;
                if let Some(secs) = settings.idle_threshold_secs {
                    idle_threshold = std::time::Duration::from_secs(secs.max(30));
                }

                if !settings.tracking_enabled {
                    if !user_idle {
                        tracker.pause_current().await?;
                        user_idle = true;
                    }
                    continue;
                }

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
                        if let Err(error) = tracker.tick(window, &settings).await {
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

async fn resolve_user_id(
    pool: &netchronicle_db::DbPool,
    config: &AgentConfig,
) -> anyhow::Result<Uuid> {
    if let Some(api_key) = &config.api_key {
        let key_hash = hash_token(api_key);
        let key = ApiKeyRepository::new(pool)
            .find_by_hash(&key_hash)
            .await?
            .ok_or_else(|| anyhow::anyhow!("AGENT_API_KEY is invalid or revoked"))?;
        let _ = ApiKeyRepository::new(pool).touch(key.id).await;
        return Ok(key.user_id);
    }

    if config.auth_required {
        anyhow::bail!("AUTH_REQUIRED=true but AGENT_API_KEY is not set");
    }

    UserRepository::new(pool)
        .get_or_create(config.user_id)
        .await
}
