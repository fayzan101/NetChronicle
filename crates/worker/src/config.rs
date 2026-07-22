use std::env;
use std::time::Duration;

use uuid::Uuid;

#[derive(Debug, Clone)]
pub struct WorkerConfig {
    pub database_url: String,
    pub user_id: Option<Uuid>,
    pub session_interval: Duration,
    pub report_interval: Duration,
    pub retention_interval: Duration,
    pub session_lookback_days: i64,
    pub report_lookback_days: i64,
    pub raw_events_retention_days: i64,
    pub run_once: bool,
}

impl WorkerConfig {
    pub fn from_env() -> Self {
        let user_id = env::var("WORKER_USER_ID")
            .ok()
            .or_else(|| env::var("AGENT_USER_ID").ok())
            .and_then(|s| Uuid::parse_str(&s).ok())
            .filter(|id| !id.is_nil());

        let database_url = env::var("DATABASE_URL").unwrap_or_else(|_| {
            "postgres://netchronicle:netchronicle@localhost:5432/netchronicle".into()
        });

        let session_interval = Duration::from_secs(
            env::var("WORKER_SESSION_INTERVAL_SECS")
                .ok()
                .and_then(|s| s.parse().ok())
                .unwrap_or(300),
        );

        let report_interval = Duration::from_secs(
            env::var("WORKER_REPORT_INTERVAL_SECS")
                .ok()
                .and_then(|s| s.parse().ok())
                .unwrap_or(900),
        );

        let retention_interval = Duration::from_secs(
            env::var("WORKER_RETENTION_INTERVAL_SECS")
                .ok()
                .and_then(|s| s.parse().ok())
                .unwrap_or(3600),
        );

        let session_lookback_days = env::var("SESSION_REBUILD_LOOKBACK_DAYS")
            .ok()
            .and_then(|s| s.parse().ok())
            .unwrap_or(2)
            .clamp(1, 14);

        let report_lookback_days = env::var("WORKER_REPORT_LOOKBACK_DAYS")
            .ok()
            .and_then(|s| s.parse().ok())
            .unwrap_or(30)
            .clamp(1, 90);

        let raw_events_retention_days = env::var("RAW_EVENTS_RETENTION_DAYS")
            .ok()
            .and_then(|s| s.parse().ok())
            .unwrap_or(30)
            .clamp(1, 365);

        let run_once = matches!(
            env::var("WORKER_RUN_ONCE")
                .unwrap_or_default()
                .to_ascii_lowercase()
                .as_str(),
            "1" | "true" | "yes" | "on"
        );

        Self {
            database_url,
            user_id,
            session_interval,
            report_interval,
            retention_interval,
            session_lookback_days,
            report_lookback_days,
            raw_events_retention_days,
            run_once,
        }
    }
}
