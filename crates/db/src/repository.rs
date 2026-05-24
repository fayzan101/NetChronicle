use chrono::{DateTime, Utc};
use sqlx::PgPool;
use uuid::Uuid;

use netchronicle_common::Session;

/// Session persistence — expand with website_logs, network_logs, reports.
pub struct SessionRepository<'a> {
    pool: &'a PgPool,
}

impl<'a> SessionRepository<'a> {
    pub fn new(pool: &'a PgPool) -> Self {
        Self { pool }
    }

    pub async fn list_for_user(
        &self,
        user_id: Uuid,
        from: DateTime<Utc>,
        to: DateTime<Utc>,
    ) -> anyhow::Result<Vec<Session>> {
        let _ = (user_id, from, to, self.pool);
        // TODO: map rows from `sessions` table to `Session`
        Ok(vec![])
    }
}
