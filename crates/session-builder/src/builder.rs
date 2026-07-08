use chrono::{DateTime, Utc};
use netchronicle_common::{ActivityCategory, AppActivityEvent, NetworkStability, SessionDraft};
use std::collections::HashMap;
use uuid::Uuid;

use crate::network::network_stability_for_window;
use crate::SessionBuilderConfig;

/// App log with database id for session linking.
#[derive(Debug, Clone)]
pub struct TrackedAppLog {
    pub log_id: Uuid,
    pub event: AppActivityEvent,
}

#[derive(Debug, Clone)]
pub struct NetworkObservation {
    pub stability: NetworkStability,
    pub disconnect: bool,
    pub recorded_at: DateTime<Utc>,
}

#[derive(Debug, Clone)]
pub struct BuiltSession {
    pub draft: SessionDraft,
    pub log_ids: Vec<Uuid>,
}

pub struct SessionBuilder {
    config: SessionBuilderConfig,
    user_id: Uuid,
}

impl SessionBuilder {
    pub fn new(user_id: Uuid, config: SessionBuilderConfig) -> Self {
        Self { config, user_id }
    }

    pub fn build_from_logs(
        &self,
        logs: &[TrackedAppLog],
        network: &[NetworkObservation],
    ) -> Vec<BuiltSession> {
        if logs.is_empty() {
            return vec![];
        }

        let idle_gap = self.config.idle_gap.as_secs() as i64;
        let min_duration = self.config.min_session_duration.as_secs() as i64;

        let mut sorted: Vec<&TrackedAppLog> = logs.iter().collect();
        sorted.sort_by_key(|log| log.event.recorded_at);

        let mut groups: Vec<Vec<&TrackedAppLog>> = Vec::new();
        let mut current: Vec<&TrackedAppLog> = Vec::new();
        let mut last_end: Option<DateTime<Utc>> = None;

        for log in sorted {
            let end = log.event.recorded_at;
            let start = end - chrono::Duration::seconds(log.event.duration_sec as i64);

            if let Some(prev_end) = last_end {
                let gap = start.signed_duration_since(prev_end).num_seconds();
                if gap > idle_gap {
                    groups.push(current);
                    current = Vec::new();
                }
            }

            current.push(log);
            last_end = Some(end);
        }

        if !current.is_empty() {
            groups.push(current);
        }

        groups
            .into_iter()
            .filter_map(|group| self.build_group(group, network, min_duration))
            .collect()
    }

    fn build_group(
        &self,
        group: Vec<&TrackedAppLog>,
        network: &[NetworkObservation],
        min_duration: i64,
    ) -> Option<BuiltSession> {
        if group.is_empty() {
            return None;
        }

        let first_start = group
            .first()
            .map(|log| {
                log.event.recorded_at
                    - chrono::Duration::seconds(log.event.duration_sec as i64)
            })?;
        let last_end = group.last()?.event.recorded_at;
        let duration = last_end.signed_duration_since(first_start).num_seconds();

        if duration < min_duration {
            return None;
        }

        let mut app_durations: HashMap<String, i64> = HashMap::new();
        let mut category_durations: HashMap<ActivityCategory, i64> = HashMap::new();
        let mut productive_sec = 0i64;
        let mut total_sec = 0i64;

        for log in &group {
            let secs = log.event.duration_sec as i64;
            total_sec += secs;
            *app_durations.entry(log.event.app_name.clone()).or_default() += secs;
            *category_durations.entry(log.event.category).or_default() += secs;

            if matches!(
                log.event.category,
                ActivityCategory::Work | ActivityCategory::Learning
            ) {
                productive_sec += secs;
            }
        }

        let mut primary_apps: Vec<(String, i64)> = app_durations.into_iter().collect();
        primary_apps.sort_by(|a, b| b.1.cmp(&a.1));
        let primary_apps: Vec<String> = primary_apps.into_iter().map(|(name, _)| name).take(5).collect();

        let category = category_durations
            .into_iter()
            .max_by_key(|(_, secs)| *secs)
            .map(|(cat, _)| cat)
            .unwrap_or(ActivityCategory::Unknown);

        let productivity_score = if total_sec > 0 {
            Some((productive_sec as f32 / total_sec as f32) * 100.0)
        } else {
            None
        };

        let network_stability =
            network_stability_for_window(network, first_start, last_end);

        let log_ids = group.iter().map(|log| log.log_id).collect();

        Some(BuiltSession {
            draft: SessionDraft {
                user_id: self.user_id,
                start_time: first_start,
                end_time: Some(last_end),
                primary_apps,
                category,
                productivity_score,
                network_stability: Some(network_stability),
            },
            log_ids,
        })
    }
}

#[cfg(test)]
mod tests {
    use super::*;
    use chrono::TimeZone;
    use netchronicle_common::ActivityCategory;

    fn log(id: Uuid, app: &str, secs: i64, at: DateTime<Utc>, category: ActivityCategory) -> TrackedAppLog {
        TrackedAppLog {
            log_id: id,
            event: AppActivityEvent {
                app_name: app.into(),
                window_title: None,
                duration_sec: secs as u32,
                category,
                recorded_at: at,
            },
        }
    }

    #[test]
    fn groups_logs_by_idle_gap() {
        let base = Utc.with_ymd_and_hms(2026, 1, 1, 9, 0, 0).unwrap();
        let logs = vec![
            log(Uuid::new_v4(), "Code", 120, base + chrono::Duration::minutes(2), ActivityCategory::Work),
            log(
                Uuid::new_v4(),
                "Code",
                180,
                base + chrono::Duration::minutes(10),
                ActivityCategory::Work,
            ),
            log(
                Uuid::new_v4(),
                "Chrome",
                60,
                base + chrono::Duration::minutes(20),
                ActivityCategory::Distraction,
            ),
        ];

        let builder = SessionBuilder::new(
            Uuid::new_v4(),
            SessionBuilderConfig {
                idle_gap: std::time::Duration::from_secs(300),
                min_session_duration: std::time::Duration::from_secs(60),
            },
        );

        let sessions = builder.build_from_logs(&logs, &[]);
        assert_eq!(sessions.len(), 2);
        assert_eq!(sessions[0].log_ids.len(), 2);
    }
}
