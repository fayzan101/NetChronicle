use std::env;
use std::time::Duration;

use uuid::Uuid;

#[derive(Debug, Clone)]
pub struct AgentConfig {
    pub user_id: Uuid,
    pub database_url: String,
    pub poll_interval: Duration,
    pub network_sample_interval: Duration,
    pub min_segment_secs: u32,
    pub session_rebuild_interval: Duration,
    pub rules_refresh_interval: Duration,
    pub idle_threshold: Duration,
    pub browser_feed_port: u16,
    pub ignore_apps: Vec<String>,
}

impl AgentConfig {
    pub fn from_env() -> anyhow::Result<Self> {
        let user_id = env::var("AGENT_USER_ID")
            .ok()
            .and_then(|s| Uuid::parse_str(&s).ok())
            .unwrap_or_else(Uuid::nil);

        let database_url = env::var("DATABASE_URL").unwrap_or_else(|_| {
            "postgres://netchronicle:netchronicle@localhost:5432/netchronicle".into()
        });

        let poll_interval = Duration::from_secs(
            env::var("AGENT_POLL_INTERVAL_SECS")
                .ok()
                .and_then(|s| s.parse().ok())
                .unwrap_or(2),
        );

        let network_sample_interval = Duration::from_secs(
            env::var("NETWORK_SAMPLE_INTERVAL_SECS")
                .ok()
                .and_then(|s| s.parse().ok())
                .unwrap_or(30),
        );

        let min_segment_secs = env::var("AGENT_MIN_SEGMENT_SECS")
            .ok()
            .and_then(|s| s.parse().ok())
            .unwrap_or(3);

        let session_rebuild_interval = Duration::from_secs(
            env::var("SESSION_REBUILD_INTERVAL_SECS")
                .ok()
                .and_then(|s| s.parse().ok())
                .unwrap_or(300),
        );

        let rules_refresh_interval = Duration::from_secs(
            env::var("RULES_REFRESH_INTERVAL_SECS")
                .ok()
                .and_then(|s| s.parse().ok())
                .unwrap_or(60),
        );

        let idle_threshold = Duration::from_secs(
            env::var("AGENT_IDLE_THRESHOLD_SECS")
                .ok()
                .and_then(|s| s.parse().ok())
                .unwrap_or(300),
        );

        let browser_feed_port = env::var("AGENT_BROWSER_FEED_PORT")
            .ok()
            .and_then(|s| s.parse().ok())
            .unwrap_or(9477);

        let ignore_apps = crate::ignore::ignore_list_from_env();

        Ok(Self {
            user_id,
            database_url,
            poll_interval,
            network_sample_interval,
            min_segment_secs,
            session_rebuild_interval,
            rules_refresh_interval,
            idle_threshold,
            browser_feed_port,
            ignore_apps,
        })
    }
}
