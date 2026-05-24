use netchronicle_common::{AppActivityEvent, SessionDraft, WebsiteVisitEvent};
use uuid::Uuid;

use crate::SessionBuilderConfig;

/// Aggregates website and app events into `SessionDraft` records.
pub struct SessionBuilder {
    config: SessionBuilderConfig,
    user_id: Uuid,
}

impl SessionBuilder {
    pub fn new(user_id: Uuid, config: SessionBuilderConfig) -> Self {
        Self { config, user_id }
    }

    pub fn build_from_events(
        &self,
        websites: &[WebsiteVisitEvent],
        apps: &[AppActivityEvent],
    ) -> Vec<SessionDraft> {
        let _ = (&self.config, websites, apps);
        // TODO: idle-gap grouping, primary app detection, network overlay
        vec![]
    }
}
