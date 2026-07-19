use chrono::NaiveDate;
use chrono::Timelike;
use netchronicle_common::{ActivityCategory, DailySummary, Session};
use serde::Serialize;

use crate::Insight;

#[derive(Debug, Clone)]
pub struct DailyAnalyticsInput {
    pub date: NaiveDate,
    pub sessions: Vec<Session>,
    pub network_health_score: f32,
}

#[derive(Debug, Clone, Serialize)]
pub struct WeeklySummary {
    pub total_online_minutes: i64,
    pub productive_minutes: i64,
    pub session_count: usize,
    pub average_productivity_score: f32,
    pub category_minutes: Vec<CategoryMinutes>,
}

#[derive(Debug, Clone, Serialize)]
pub struct CategoryMinutes {
    pub category: String,
    pub minutes: i64,
}

pub struct AnalyticsEngine;

impl AnalyticsEngine {
    pub fn daily_summary(input: &DailyAnalyticsInput) -> DailySummary {
        let mut total_sec = 0i64;
        let mut productive_sec = 0i64;
        let mut distraction_sec = 0i64;

        for session in &input.sessions {
            let Some(end) = session.end_time else {
                continue;
            };
            let secs = end
                .signed_duration_since(session.start_time)
                .num_seconds()
                .max(0);
            total_sec += secs;

            match session.category {
                ActivityCategory::Work | ActivityCategory::Learning => productive_sec += secs,
                ActivityCategory::Distraction => distraction_sec += secs,
                _ => {}
            }
        }

        let productivity_score = if total_sec > 0 {
            (productive_sec as f32 / total_sec as f32) * 100.0
        } else {
            0.0
        };

        let distraction_ratio = if total_sec > 0 {
            distraction_sec as f32 / total_sec as f32
        } else {
            0.0
        };

        DailySummary {
            date: input.date,
            productivity_score,
            total_online_minutes: (total_sec / 60) as u32,
            network_health_score: input.network_health_score,
            distraction_ratio,
            focus_minutes: (productive_sec / 60) as u32,
        }
    }

    pub fn weekly_summary(sessions: &[Session]) -> WeeklySummary {
        let mut total_sec = 0i64;
        let mut productive_sec = 0i64;
        let mut score_sum = 0.0f32;
        let mut score_count = 0usize;
        let mut category_sec: std::collections::HashMap<ActivityCategory, i64> =
            std::collections::HashMap::new();

        for session in sessions {
            let Some(end) = session.end_time else {
                continue;
            };
            let secs = end
                .signed_duration_since(session.start_time)
                .num_seconds()
                .max(0);
            total_sec += secs;
            *category_sec.entry(session.category).or_default() += secs;

            if matches!(
                session.category,
                ActivityCategory::Work | ActivityCategory::Learning
            ) {
                productive_sec += secs;
            }

            if let Some(score) = session.productivity_score {
                score_sum += score;
                score_count += 1;
            }
        }

        let mut category_minutes: Vec<CategoryMinutes> = category_sec
            .into_iter()
            .map(|(category, secs)| CategoryMinutes {
                category: category_label(category).into(),
                minutes: secs / 60,
            })
            .collect();
        category_minutes.sort_by(|a, b| b.minutes.cmp(&a.minutes));

        WeeklySummary {
            total_online_minutes: total_sec / 60,
            productive_minutes: productive_sec / 60,
            session_count: sessions.len(),
            average_productivity_score: if score_count > 0 {
                score_sum / score_count as f32
            } else {
                0.0
            },
            category_minutes,
        }
    }

    pub fn generate_insights(
        sessions: &[Session],
        top_apps: &[(String, i64)],
        top_domains: &[(String, i64)],
    ) -> Vec<Insight> {
        let mut insights = Vec::new();

        if sessions.is_empty() {
            insights.push(Insight {
                title: "Start tracking".into(),
                body: "Run the NetChronicle agent to begin collecting activity data.".into(),
                severity: crate::InsightSeverity::Info,
            });
            return insights;
        }

        let summary = Self::weekly_summary(sessions);
        let total = summary.total_online_minutes.max(1);
        let distraction_minutes = summary
            .category_minutes
            .iter()
            .find(|row| row.category == "distraction")
            .map(|row| row.minutes)
            .unwrap_or(0);
        let distraction_pct = (distraction_minutes as f32 / total as f32) * 100.0;

        if distraction_pct > 20.0 {
            insights.push(Insight {
                title: "High distraction time".into(),
                body: format!(
                    "Distraction activity accounted for {:.0}% of tracked session time.",
                    distraction_pct
                ),
                severity: crate::InsightSeverity::Warning,
            });
        }

        let productive_pct = (summary.productive_minutes as f32 / total as f32) * 100.0;
        if productive_pct >= 60.0 {
            insights.push(Insight {
                title: "Strong focus".into(),
                body: format!(
                    "{:.0}% of your session time was work or learning.",
                    productive_pct
                ),
                severity: crate::InsightSeverity::Positive,
            });
        }

        if let Some((app, secs)) = top_apps.first() {
            insights.push(Insight {
                title: "Most used app".into(),
                body: format!("You spent {} minutes in {}.", secs / 60, app),
                severity: crate::InsightSeverity::Info,
            });
        }

        if let Some((domain, secs)) = top_domains.first() {
            insights.push(Insight {
                title: "Top website".into(),
                body: format!("{} was your most visited site ({} minutes).", domain, secs / 60),
                severity: crate::InsightSeverity::Info,
            });
        }

        let unstable = sessions
            .iter()
            .filter(|s| {
                s.network_stability == Some(netchronicle_common::NetworkStability::Unstable)
                    || s.network_stability == Some(netchronicle_common::NetworkStability::Offline)
            })
            .count();
        if unstable > 0 {
            insights.push(Insight {
                title: "Network affected sessions".into(),
                body: format!(
                    "{unstable} session(s) overlapped with unstable or offline network conditions."
                ),
                severity: crate::InsightSeverity::Warning,
            });
        }

        // Richer network ↔ focus correlation
        let degraded_focus: Vec<_> = sessions
            .iter()
            .filter(|s| {
                matches!(
                    s.category,
                    ActivityCategory::Work | ActivityCategory::Learning
                ) && matches!(
                    s.network_stability,
                    Some(netchronicle_common::NetworkStability::Degraded)
                        | Some(netchronicle_common::NetworkStability::Unstable)
                        | Some(netchronicle_common::NetworkStability::Offline)
                )
            })
            .collect();

        if let Some(session) = degraded_focus.first() {
            let label = match session.network_stability {
                Some(netchronicle_common::NetworkStability::Offline) => "offline",
                Some(netchronicle_common::NetworkStability::Unstable) => "unstable",
                _ => "degraded",
            };
            let hour = session.start_time.hour();
            let apps = if session.primary_apps.is_empty() {
                "a focus session".to_string()
            } else {
                session.primary_apps[0].clone()
            };
            insights.push(Insight {
                title: "Network hurt focus time".into(),
                body: format!(
                    "Network was {label} during your {apps} session around {hour:02}:00 — quality may have reduced productivity."
                ),
                severity: crate::InsightSeverity::Warning,
            });
        }

        insights
    }
}

fn category_label(category: ActivityCategory) -> &'static str {
    match category {
        ActivityCategory::Work => "work",
        ActivityCategory::Learning => "learning",
        ActivityCategory::Entertainment => "entertainment",
        ActivityCategory::Distraction => "distraction",
        ActivityCategory::Neutral => "neutral",
        ActivityCategory::Unknown => "unknown",
    }
}

#[cfg(test)]
mod tests {
    use super::*;
    use chrono::{TimeZone, Utc};
    use netchronicle_common::ActivityCategory;
    use uuid::Uuid;

    #[test]
    fn daily_summary_from_sessions() {
        let start = Utc.with_ymd_and_hms(2026, 1, 1, 9, 0, 0).unwrap();
        let sessions = vec![Session {
            session_id: Uuid::new_v4(),
            user_id: Uuid::new_v4(),
            start_time: start,
            end_time: Some(start + chrono::Duration::minutes(30)),
            category: ActivityCategory::Work,
            productivity_score: Some(90.0),
            network_stability: None,
            primary_apps: vec!["Code".into()],
        }];

        let summary = AnalyticsEngine::daily_summary(&DailyAnalyticsInput {
            date: start.date_naive(),
            sessions,
            network_health_score: 95.0,
        });

        assert_eq!(summary.total_online_minutes, 30);
        assert_eq!(summary.focus_minutes, 30);
    }

    #[test]
    fn network_focus_insight() {
        let start = Utc.with_ymd_and_hms(2026, 1, 1, 14, 0, 0).unwrap();
        let sessions = vec![Session {
            session_id: Uuid::new_v4(),
            user_id: Uuid::new_v4(),
            start_time: start,
            end_time: Some(start + chrono::Duration::minutes(45)),
            category: ActivityCategory::Work,
            productivity_score: Some(80.0),
            network_stability: Some(netchronicle_common::NetworkStability::Unstable),
            primary_apps: vec!["Code".into()],
        }];

        let insights = AnalyticsEngine::generate_insights(&sessions, &[], &[]);
        assert!(insights.iter().any(|i| i.title.contains("Network hurt focus")));
        assert!(insights
            .iter()
            .any(|i| i.body.contains("14:00") && i.body.contains("Code")));
    }
}
