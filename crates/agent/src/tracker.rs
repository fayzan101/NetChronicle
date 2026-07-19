use std::time::Duration;

use chrono::{DateTime, Utc};
use netchronicle_common::ActivityCategory;
use netchronicle_db::{category_to_db, ActivityRepository, DbPool};
use serde_json::json;
use tracing::debug;
use uuid::Uuid;

use crate::browser::{browser_url_proxy, is_browser, parse_browser_context, BrowserContext};
use crate::browser_feed::BrowserFeed;
use crate::db_retry::with_db_retry;
use crate::rules_cache::RulesCache;
use crate::window::ForegroundWindow;

#[derive(Debug, Clone)]
struct ActiveSegment {
    window: ForegroundWindow,
    browser: Option<BrowserContext>,
    category: ActivityCategory,
    started_at: DateTime<Utc>,
    duration_sec: u32,
}

pub struct ActivityTracker {
    user_id: Uuid,
    pool: DbPool,
    rules: RulesCache,
    browser_feed: BrowserFeed,
    min_segment_secs: u32,
    poll_secs: u32,
    last_heartbeat: Option<DateTime<Utc>>,
    current: Option<ActiveSegment>,
}

impl ActivityTracker {
    pub fn new(
        user_id: Uuid,
        pool: DbPool,
        rules: RulesCache,
        browser_feed: BrowserFeed,
        min_segment_secs: u32,
        poll_secs: u32,
    ) -> Self {
        Self {
            user_id,
            pool,
            rules,
            browser_feed,
            min_segment_secs,
            poll_secs,
            last_heartbeat: None,
            current: None,
        }
    }

    pub async fn tick(&mut self, window: ForegroundWindow) -> anyhow::Result<()> {
        let browser = self.resolve_browser_context(&window).await;
        let domain = browser.as_ref().and_then(|ctx| ctx.domain.clone());
        let url = browser.as_ref().map(|ctx| ctx.url.as_str());

        let category = self
            .rules
            .classify_activity(&window.app_name, url, domain.as_deref())
            .await;

        let same_segment = self.current.as_ref().is_some_and(|segment| {
            segment.window.app_name == window.app_name
                && segment.window.window_title == window.window_title
                && segment.browser == browser
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
            browser,
            category,
            started_at: Utc::now(),
            duration_sec: self.poll_secs,
        });
        self.publish_snapshot(false).await?;

        Ok(())
    }

    async fn resolve_browser_context(
        &self,
        window: &ForegroundWindow,
    ) -> Option<BrowserContext> {
        if !is_browser(&window.app_name) {
            return None;
        }

        if let Some(tab) = self
            .browser_feed
            .latest_fresh(Duration::from_secs(30))
            .await
        {
            let domain = crate::browser::extract_domain_from_url(&tab.url);
            return Some(BrowserContext {
                page_title: tab.title.unwrap_or_else(|| window.window_title.clone()),
                domain,
                url: tab.url,
            });
        }

        if let Ok(Some(row)) = ActivityRepository::new(&self.pool)
            .latest_browser_tab(self.user_id)
            .await
        {
            let age = Utc::now().signed_duration_since(row.recorded_at);
            if age.num_seconds() <= 30 {
                let url = row
                    .payload
                    .get("url")
                    .and_then(|v| v.as_str())
                    .unwrap_or_default()
                    .to_string();
                if !url.is_empty() {
                    let title = row
                        .payload
                        .get("title")
                        .and_then(|v| v.as_str())
                        .map(str::to_string);
                    let domain = crate::browser::extract_domain_from_url(&url);
                    return Some(BrowserContext {
                        page_title: title.unwrap_or_else(|| window.window_title.clone()),
                        domain,
                        url,
                    });
                }
            }
        }

        parse_browser_context(&window.app_name, &window.window_title)
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
            self.snapshot_payload(segment, heartbeat, false),
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
                app = %segment.window.friendly_name,
                secs = segment.duration_sec,
                "skipping short activity segment"
            );
            return Ok(());
        }

        let recorded_at = Utc::now();
        let repo = ActivityRepository::new(&self.pool);
        let display_name = &segment.window.friendly_name;

        with_db_retry(|| async {
            repo.insert_app_log(
                self.user_id,
                display_name,
                Some(&segment.window.window_title),
                segment.duration_sec as i32,
                segment.category,
                recorded_at,
            )
            .await
        })
        .await?;

        if let Some(browser) = &segment.browser {
            if let Some(domain) = &browser.domain {
                let url = browser_url_proxy(browser);
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
        }

        with_db_retry(|| async {
            repo.insert_raw_event(
                self.user_id,
                "activity_snapshot",
                self.snapshot_payload(&segment, false, true),
                recorded_at,
            )
            .await
        })
        .await?;

        with_db_retry(|| async {
            repo.insert_raw_event(
                self.user_id,
                "app_metadata",
                json!({
                    "app": segment.window.app_name,
                    "friendlyName": segment.window.friendly_name,
                    "processPath": segment.window.process_path,
                    "processId": segment.window.process_id,
                    "windowTitle": segment.window.window_title,
                    "durationSec": segment.duration_sec,
                }),
                recorded_at,
            )
            .await
        })
        .await?;

        self.last_heartbeat = None;

        debug!(
            app = %segment.window.friendly_name,
            domain = ?segment.browser.as_ref().and_then(|b| b.domain.clone()),
            secs = segment.duration_sec,
            "flushed activity segment"
        );

        Ok(())
    }

    fn snapshot_payload(
        &self,
        segment: &ActiveSegment,
        heartbeat: bool,
        flushed: bool,
    ) -> serde_json::Value {
        json!({
            "app": segment.window.friendly_name,
            "appExec": segment.window.app_name,
            "title": segment.window.window_title,
            "domain": segment.browser.as_ref().and_then(|b| b.domain.clone()),
            "url": segment.browser.as_ref().map(|b| b.url.clone()).unwrap_or_default(),
            "category": category_to_db(segment.category),
            "durationSec": segment.duration_sec,
            "startedAt": segment.started_at.to_rfc3339(),
            "processPath": segment.window.process_path,
            "processId": segment.window.process_id,
            "heartbeat": heartbeat,
            "flushed": flushed,
        })
    }

    pub async fn pause_current(&mut self) -> anyhow::Result<()> {
        self.flush_current().await
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
    use netchronicle_network_monitor::{CompositeProbe, NetworkProbe};
    use tracing::warn;

    let probe = CompositeProbe::from_env();
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
