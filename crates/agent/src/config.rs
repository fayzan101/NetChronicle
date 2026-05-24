use std::env;
use std::time::Duration;

use uuid::Uuid;

#[derive(Debug, Clone)]
pub struct AgentConfig {
    pub user_id: Uuid,
    pub database_url: String,
    pub network_sample_interval: Duration,
}

impl AgentConfig {
    pub fn from_env() -> anyhow::Result<Self> {
        let user_id = env::var("AGENT_USER_ID")
            .ok()
            .and_then(|s| Uuid::parse_str(&s).ok())
            .unwrap_or_else(Uuid::nil);

        let database_url = env::var("DATABASE_URL")
            .unwrap_or_else(|_| "postgres://netchronicle:netchronicle@localhost:5432/netchronicle".into());

        let network_sample_interval = Duration::from_secs(
            env::var("NETWORK_SAMPLE_INTERVAL_SECS")
                .ok()
                .and_then(|s| s.parse().ok())
                .unwrap_or(30),
        );

        Ok(Self {
            user_id,
            database_url,
            network_sample_interval,
        })
    }
}
