mod config;
mod reports;
mod retention;
mod sessions;

use chrono::Utc;
use netchronicle_db::{create_pool, run_migrations, UserRepository};
use tracing::info;
use tracing_subscriber::EnvFilter;
use uuid::Uuid;

use crate::config::WorkerConfig;

#[tokio::main]
async fn main() -> anyhow::Result<()> {
    dotenvy::dotenv().ok();
    tracing_subscriber::fmt()
        .with_env_filter(EnvFilter::from_default_env().add_directive("info".parse()?))
        .init();

    let config = WorkerConfig::from_env();
    info!(
        session_interval_secs = config.session_interval.as_secs(),
        report_interval_secs = config.report_interval.as_secs(),
        retention_interval_secs = config.retention_interval.as_secs(),
        session_lookback_days = config.session_lookback_days,
        report_lookback_days = config.report_lookback_days,
        raw_events_retention_days = config.raw_events_retention_days,
        run_once = config.run_once,
        "starting netchronicle-worker"
    );

    let pool = create_pool(&config.database_url).await?;
    run_migrations(&pool).await?;

    if config.run_once {
        run_all_jobs(&pool, &config).await?;
        return Ok(());
    }

    let session_pool = pool.clone();
    let report_pool = pool.clone();
    let retention_pool = pool.clone();
    let session_cfg = config.clone();
    let report_cfg = config.clone();
    let retention_cfg = config.clone();

    let sessions = tokio::spawn(async move {
        let mut ticker = tokio::time::interval(session_cfg.session_interval);
        loop {
            ticker.tick().await;
            if let Err(error) = run_session_jobs(&session_pool, &session_cfg).await {
                tracing::warn!(%error, "session job failed");
            }
        }
    });

    let reports = tokio::spawn(async move {
        let mut ticker = tokio::time::interval(report_cfg.report_interval);
        loop {
            ticker.tick().await;
            if let Err(error) = run_report_jobs(&report_pool, &report_cfg).await {
                tracing::warn!(%error, "report job failed");
            }
        }
    });

    let retention = tokio::spawn(async move {
        let mut ticker = tokio::time::interval(retention_cfg.retention_interval);
        loop {
            ticker.tick().await;
            if let Err(error) = run_retention_jobs(&retention_pool, &retention_cfg).await {
                tracing::warn!(%error, "retention job failed");
            }
        }
    });

    tokio::select! {
        r = sessions => r?,
        r = reports => r?,
        r = retention => r?,
    }

    Ok(())
}

async fn resolve_users(
    pool: &netchronicle_db::DbPool,
    preferred: Option<Uuid>,
) -> anyhow::Result<Vec<Uuid>> {
    if let Some(id) = preferred {
        return Ok(vec![UserRepository::new(pool).get_or_create(id).await?]);
    }

    let mut ids = UserRepository::new(pool).list_ids().await?;
    if ids.is_empty() {
        ids.push(UserRepository::new(pool).ensure_local_user().await?);
    }
    Ok(ids)
}

async fn run_all_jobs(pool: &netchronicle_db::DbPool, config: &WorkerConfig) -> anyhow::Result<()> {
    run_session_jobs(pool, config).await?;
    run_report_jobs(pool, config).await?;
    run_retention_jobs(pool, config).await?;
    Ok(())
}

async fn run_session_jobs(
    pool: &netchronicle_db::DbPool,
    config: &WorkerConfig,
) -> anyhow::Result<()> {
    let today = Utc::now().date_naive();
    for user_id in resolve_users(pool, config.user_id).await? {
        sessions::rebuild_sessions_for_lookback(user_id, pool, today, config.session_lookback_days)
            .await?;
    }
    Ok(())
}

async fn run_report_jobs(
    pool: &netchronicle_db::DbPool,
    config: &WorkerConfig,
) -> anyhow::Result<()> {
    let today = Utc::now().date_naive();
    for user_id in resolve_users(pool, config.user_id).await? {
        reports::compute_reports_for_user(user_id, pool, today, config.report_lookback_days)
            .await?;
    }
    Ok(())
}

async fn run_retention_jobs(
    pool: &netchronicle_db::DbPool,
    config: &WorkerConfig,
) -> anyhow::Result<()> {
    if let Some(user_id) = config.user_id {
        retention::prune_raw_events(pool, Some(user_id), config.raw_events_retention_days).await?;
    } else {
        retention::prune_raw_events(pool, None, config.raw_events_retention_days).await?;
    }
    Ok(())
}
