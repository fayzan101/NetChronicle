use std::sync::Arc;
use std::time::Duration;

use netchronicle_db::{DbPool, UserRepository, UserSettings};
use tokio::sync::RwLock;
use tracing::{debug, info, warn};
use uuid::Uuid;

#[derive(Clone)]
pub struct SettingsCache {
    inner: Arc<RwLock<UserSettings>>,
}

impl SettingsCache {
    pub fn new(settings: UserSettings) -> Self {
        Self {
            inner: Arc::new(RwLock::new(settings)),
        }
    }

    pub async fn get(&self) -> UserSettings {
        self.inner.read().await.clone()
    }

    pub fn spawn_refresh(self, user_id: Uuid, pool: DbPool, interval: Duration) {
        tokio::spawn(async move {
            let mut ticker = tokio::time::interval(interval);
            loop {
                ticker.tick().await;
                match UserRepository::new(&pool).get_settings(user_id).await {
                    Ok(settings) => {
                        let mut guard = self.inner.write().await;
                        if *guard != settings {
                            info!(
                                tracking = settings.tracking_enabled,
                                "settings refreshed from database"
                            );
                            *guard = settings;
                        } else {
                            debug!("settings unchanged");
                        }
                    }
                    Err(error) => warn!(%error, "failed to refresh settings"),
                }
            }
        });
    }
}

// UserSettings needs PartialEq - add to db crate
