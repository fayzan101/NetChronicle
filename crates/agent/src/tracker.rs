use chrono::{DateTime, Utc};
use netchronicle_categorization::Categorizer;
use netchronicle_common::ActivityCategory;
use netchronicle_db::{category_to_db, ActivityRepository, DbPool};
use serde_json::json;
use tracing::{debug, warn};
use uuid::Uuid;

use crate::browser::{browser_url_proxy, extract_domain, is_browser};
use crate::db_retry::with_db_retry;
use crate::window::ForegroundWindow;

#[derive(Debug, Clone)]
struct ActiveSegment {
    window: ForegroundWindow,
    domain: Option<String>,
    category: ActivityCategory,
    started_at: DateTime<Utc>,
    duration_sec: u32,
}

pub struct ActivityTracker {
    user_id: Uuid,
    pool: DbPool,
    categorizer: Categorizer,
    min_segment_secs: u32,
    poll_secs: u32,
    last_heartbeat: Option<DateTime<Utc>>,
    current: Option<ActiveSegment>,
}

impl ActivityTracker {
    pub fn new(
        user_id: Uuid,
        pool: DbPool,
        categorizer: Categorizer,
        min_segment_secs: u32,
        poll_secs: u32,
    ) -> Self {
        Self {
            user_id,
            pool,
            categorizer,
            min_segment_secs,
            poll_secs,
            last_heartbeat: None,
            current: None,
        }
    }

    pub async fn tick(&mut self, window: ForegroundWindow) -> anyhow::Result<()> {
        let domain = if is_browser(&window.app_name) {
            extract_domain(&window.window_title)
        } else {
            None
        };
        let category = self.categorizer.classify_activity(
            &window.app_name,
            domain.as_deref(),
        );

        let same_segment = self.current.as_ref().is_some_and(|segment| {
            segment.window.app_name == window.app_name
                && segment.window.window_title == window.window_title
                && segment.domain == domain
        });

        if same_segment {
            if let Some(segment) = &mut self.current {
                segment.duration_sec += self.poll_secs;
            }
            self.maybe_heartbeat().await?;
            return Ok(());
        }

        self.flush_current().await?;

        self.current = Some(ActiveSegment {
            window,
            domain,
            category,
            started_at: Utc::now(),
            duration_sec: self.poll_secs,
        });
        self.publish_snapshot(false).await?;

        Ok(())
    }

    async fn maybe_heartbeat(&mut self) -> anyhow::Result<()> {
        let should_publish = self.last_heartbeat.is_none_or(|last| {
            Utc::now().signed_duration_since(last).num_seconds() >= 10
        });

        if should_publish {
            self.publish_snapshot(true).await?;
        }

        Ok(())
    }

    async fn publish_snapshot(&mut self, heartbeat: bool) -> anyhow::Result<()> {
        let Some(segment) = &self.current else {
            return Ok(());
        };

        let recorded_at = Utc::now();
        let repo = ActivityRepository::new(&self.pool);
        repo.insert_raw_event(
            self.user_id,
            "activity_snapshot",
            json!({
                "app": segment.window.app_name,
                "title": segment.window.window_title,
                "domain": segment.domain,
                "category": category_to_db(segment.category),
                "duration_sec": segment.duration_sec,
                "started_at": segment.started_at.to_rfc3339(),
                "heartbeat": heartbeat,
            }),
            recorded_at,
        )
        .await?;

        self.last_heartbeat = Some(recorded_at);
        Ok(())
    }

    pub async fn flush_current(&mut self) -> anyhow::Result<()> {
        let Some(segment) = self.current.take() else {
            return Ok(());
        };

        if segment.duration_sec < self.min_segment_secs {
            debug!(
                app = %segment.window.app_name,
                secs = segment.duration_sec,
                "skipping short activity segment"
            );
            return Ok(());
        }

        let recorded_at = Utc::now();
        let repo = ActivityRepository::new(&self.pool);

        with_db_retry(|| async {
            repo.insert_app_log(
                self.user_id,
                &segment.window.app_name,
                Some(&segment.window.window_title),
                segment.duration_sec as i32,
                segment.category,
                recorded_at,
            )
            .await
        })
        .await?;

        if let Some(domain) = &segment.domain {
            let url = browser_url_proxy(&segment.window.window_title, domain);
            with_db_retry(|| async {
                repo.insert_website_log(
                    self.user_id,
                    &url,
                    domain,
                    segment.duration_sec as i32,
                    segment.category,
                    recorded_at,
                )
                .await
            })
            .await?;
        }

        with_db_retry(|| async {
            repo.insert_raw_event(
                self.user_id,
                "activity_snapshot",
                json!({
                    "app": segment.window.app_name,
                    "title": segment.window.window_title,
                    "domain": segment.domain,
                    "category": category_to_db(segment.category),
                    "duration_sec": segment.duration_sec,
                    "started_at": segment.started_at.to_rfc3339(),
                    "flushed": true,
                }),
                recorded_at,
            )
            .await
        })
        .await?;

        self.last_heartbeat = None;

        debug!(
            app = %segment.window.app_name,
            domain = ?segment.domain,
            secs = segment.duration_sec,
            "flushed activity segment"
        );

        Ok(())
    }

    pub async fn shutdown(&mut self) -> anyhow::Result<()> {
        self.flush_current().await
    }
}

pub async fn run_network_sampler(
    user_id: Uuid,
    pool: DbPool,
    interval: std::time::Duration,
) {
    use netchronicle_db::NetworkRepository;
    use netchronicle_network_monitor::{NetworkProbe, TcpProbe};

    let probe = TcpProbe::default();
    let mut ticker = tokio::time::interval(interval);

    loop {
        ticker.tick().await;
        let sample = probe.sample().await;
        let repo = NetworkRepository::new(&pool);

        if let Err(error) = repo
            .insert_log(
                user_id,
                sample.latency_ms,
                sample.packet_loss_pct,
                sample.bandwidth_mbps,
                sample.stability,
                sample.disconnect,
                sample.recorded_at,
            )
            .await
        {
            warn!(%error, "failed to persist network sample");
        }
    }
}
