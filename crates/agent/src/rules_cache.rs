use std::sync::Arc;
use std::time::Duration;

use netchronicle_categorization::{rule_store_from_db, Categorizer, RuleStore};
use netchronicle_db::{CategoryRuleRepository, DbPool};
use tokio::sync::RwLock;
use tracing::warn;
use uuid::Uuid;

#[derive(Clone)]
pub struct RulesCache {
    inner: Arc<RwLock<Categorizer>>,
}

impl RulesCache {
    pub async fn load(user_id: Uuid, pool: &DbPool) -> anyhow::Result<Self> {
        let categorizer = load_categorizer(user_id, pool).await?;
        Ok(Self {
            inner: Arc::new(RwLock::new(categorizer)),
        })
    }

    pub fn spawn_refresh(self, user_id: Uuid, pool: DbPool, interval: Duration) {
        tokio::spawn(async move {
            let mut ticker = tokio::time::interval(interval);
            loop {
                ticker.tick().await;
                match load_categorizer(user_id, &pool).await {
                    Ok(categorizer) => {
                        *self.inner.write().await = categorizer;
                    }
                    Err(error) => warn!(%error, "failed to refresh category rules"),
                }
            }
        });
    }

    pub async fn classify_activity(
        &self,
        app_name: &str,
        url: Option<&str>,
        domain: Option<&str>,
    ) -> netchronicle_common::ActivityCategory {
        self.inner
            .read()
            .await
            .classify_activity(app_name, url, domain)
    }
}

async fn load_categorizer(user_id: Uuid, pool: &DbPool) -> anyhow::Result<Categorizer> {
    let rows = CategoryRuleRepository::new(pool).list(user_id).await?;
    if rows.is_empty() {
        return Ok(Categorizer::new(RuleStore::with_defaults()));
    }
    Ok(Categorizer::new(rule_store_from_db(&rows)))
}
