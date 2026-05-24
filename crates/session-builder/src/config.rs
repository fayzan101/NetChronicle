use std::time::Duration;

/// Configuration for session grouping heuristics.
#[derive(Debug, Clone)]
pub struct SessionBuilderConfig {
    /// Gap between activities that starts a new session.
    pub idle_gap: Duration,
    /// Minimum duration for a session to be persisted.
    pub min_session_duration: Duration,
}

impl Default for SessionBuilderConfig {
    fn default() -> Self {
        Self {
            idle_gap: Duration::from_secs(5 * 60),
            min_session_duration: Duration::from_secs(60),
        }
    }
}
