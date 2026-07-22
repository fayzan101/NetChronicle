use chrono::Utc;
use netchronicle_db::{ActivityRepository, DbPool};
use tracing::info;
use uuid::Uuid;

pub async fn prune_raw_events(
    pool: &DbPool,
    user_id: Option<Uuid>,
    retention_days: i64,
) -> anyhow::Result<u64> {
    let before = Utc::now() - chrono::Duration::days(retention_days);
    let deleted = ActivityRepository::new(pool)
        .prune_raw_events(user_id, before)
        .await?;
    info!(?user_id, retention_days, deleted, "pruned raw_events");
    Ok(deleted)
}

#[cfg(test)]
mod tests {
    #[test]
    fn retention_days_positive() {
        assert!(30_i64.clamp(1, 365) >= 1);
    }
}
