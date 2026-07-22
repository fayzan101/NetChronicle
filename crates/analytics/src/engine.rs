use chrono::NaiveDate;
use chrono::Timelike;
use netchronicle_common::{ActivityCategory, DailySummary, NetworkStability, Session};
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
#[serde(rename_all = "camelCase")]
pub struct PeriodSummary {
    pub total_online_minutes: i64,
    pub productive_minutes: i64,
    pub session_count: usize,
    pub average_productivity_score: f32,
    pub category_minutes: Vec<CategoryMinutes>,
    pub distraction_impact_pct: f32,
    pub time_of_day: Vec<HourBucket>,
}

#[derive(Debug, Clone, Serialize)]
pub struct CategoryMinutes {
    pub category: String,
    pub minutes: i64,
}

#[derive(Debug, Clone, Serialize)]
#[serde(rename_all = "camelCase")]
pub struct HourBucket {
    pub hour: u32,
    pub total_minutes: i64,
    pub productive_minutes: i64,
    pub distraction_minutes: i64,
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
        let period = Self::period_totals(sessions);
        WeeklySummary {
            total_online_minutes: period.0 / 60,
            productive_minutes: period.1 / 60,
            session_count: sessions.len(),
            average_productivity_score: period.2,
            category_minutes: period.3,
        }
    }

    pub fn monthly_summary(sessions: &[Session]) -> PeriodSummary {
        PeriodSummary {
            total_online_minutes: Self::period_totals(sessions).0 / 60,
            productive_minutes: Self::period_totals(sessions).1 / 60,
            session_count: sessions.len(),
            average_productivity_score: Self::period_totals(sessions).2,
            category_minutes: Self::period_totals(sessions).3,
            distraction_impact_pct: Self::distraction_impact_pct(sessions),
            time_of_day: Self::time_of_day_patterns(sessions),
        }
    }

    /// Distraction minutes as % of all tracked session time.
    pub fn distraction_impact_pct(sessions: &[Session]) -> f32 {
        let (total_sec, _, _, categories) = Self::period_totals(sessions);
        if total_sec == 0 {
            return 0.0;
        }
        let distraction_sec = categories
            .iter()
            .find(|c| c.category == "distraction")
            .map(|c| c.minutes * 60)
            .unwrap_or(0);
        (distraction_sec as f32 / total_sec as f32) * 100.0
    }

    /// Aggregate productive vs distraction minutes by hour-of-day (0–23).
    pub fn time_of_day_patterns(sessions: &[Session]) -> Vec<HourBucket> {
        let mut totals = [0i64; 24];
        let mut productive = [0i64; 24];
        let mut distraction = [0i64; 24];

        for session in sessions {
            let Some(end) = session.end_time else {
                continue;
            };
            let secs = end
                .signed_duration_since(session.start_time)
                .num_seconds()
                .max(0);
            let hour = session.start_time.hour() as usize;
            totals[hour] += secs;
            match session.category {
                ActivityCategory::Work | ActivityCategory::Learning => productive[hour] += secs,
                ActivityCategory::Distraction => distraction[hour] += secs,
                _ => {}
            }
        }

        (0..24)
            .filter(|&h| totals[h] > 0)
            .map(|h| HourBucket {
                hour: h as u32,
                total_minutes: totals[h] / 60,
                productive_minutes: productive[h] / 60,
                distraction_minutes: distraction[h] / 60,
            })
            .collect()
    }

    fn period_totals(sessions: &[Session]) -> (i64, i64, f32, Vec<CategoryMinutes>) {
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
        category_minutes.sort_by_key(|b| std::cmp::Reverse(b.minutes));

        let avg = if score_count > 0 {
            score_sum / score_count as f32
        } else {
            0.0
        };

        (total_sec, productive_sec, avg, category_minutes)
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
        let distraction_pct = Self::distraction_impact_pct(sessions);

        if distraction_pct > 20.0 {
            insights.push(Insight {
                title: "High distraction impact".into(),
                body: format!(
                    "Distractions took {:.0}% of tracked time — cutting that by half would free ~{:.0} minutes of focus.",
                    distraction_pct,
                    (distraction_pct / 100.0) * total as f32 * 0.5
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

        // Peak productive hour
        let patterns = Self::time_of_day_patterns(sessions);
        if let Some(peak) = patterns.iter().max_by_key(|b| b.productive_minutes) {
            if peak.productive_minutes >= 20 {
                insights.push(Insight {
                    title: "Peak focus window".into(),
                    body: format!(
                        "Your most productive hour was around {hour:02}:00 ({mins} focus minutes).",
                        hour = peak.hour,
                        mins = peak.productive_minutes
                    ),
                    severity: crate::InsightSeverity::Positive,
                });
            }
        }

        // Peak distraction hour
        if let Some(peak) = patterns.iter().max_by_key(|b| b.distraction_minutes) {
            if peak.distraction_minutes >= 15 {
                insights.push(Insight {
                    title: "Distraction hotspot".into(),
                    body: format!(
                        "Distractions clustered around {hour:02}:00 ({mins} minutes) — protect that slot.",
                        hour = peak.hour,
                        mins = peak.distraction_minutes
                    ),
                    severity: crate::InsightSeverity::Warning,
                });
            }
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
                body: format!(
                    "{} was your most visited site ({} minutes).",
                    domain,
                    secs / 60
                ),
                severity: crate::InsightSeverity::Info,
            });
        }

        let unstable = sessions
            .iter()
            .filter(|s| {
                s.network_stability == Some(NetworkStability::Unstable)
                    || s.network_stability == Some(NetworkStability::Offline)
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

        let degraded_focus: Vec<_> = sessions
            .iter()
            .filter(|s| {
                matches!(
                    s.category,
                    ActivityCategory::Work | ActivityCategory::Learning
                ) && matches!(
                    s.network_stability,
                    Some(NetworkStability::Degraded)
                        | Some(NetworkStability::Unstable)
                        | Some(NetworkStability::Offline)
                )
            })
            .collect();

        if let Some(session) = degraded_focus.first() {
            let label = match session.network_stability {
                Some(NetworkStability::Offline) => "offline",
                Some(NetworkStability::Unstable) => "unstable",
                _ => "degraded",
            };
            let hour = session.start_time.hour();
            let apps = if session.primary_apps.is_empty() {
                "a focus session".to_string()
            } else {
                session.primary_apps[0].clone()
            };
            let focus_hit = degraded_focus.len();
            insights.push(Insight {
                title: "Network hurt focus time".into(),
                body: format!(
                    "Network was {label} during your {apps} session around {hour:02}:00 — {focus_hit} focus session(s) hit quality issues."
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
    use uuid::Uuid;

    fn session(
        hour: u32,
        mins: i64,
        category: ActivityCategory,
        stability: Option<NetworkStability>,
    ) -> Session {
        let start = Utc.with_ymd_and_hms(2026, 1, 1, hour, 0, 0).unwrap();
        Session {
            session_id: Uuid::new_v4(),
            user_id: Uuid::new_v4(),
            start_time: start,
            end_time: Some(start + chrono::Duration::minutes(mins)),
            category,
            productivity_score: Some(80.0),
            network_stability: stability,
            primary_apps: vec!["Code".into()],
        }
    }

    #[test]
    fn daily_summary_from_sessions() {
        let sessions = vec![session(9, 30, ActivityCategory::Work, None)];
        let summary = AnalyticsEngine::daily_summary(&DailyAnalyticsInput {
            date: sessions[0].start_time.date_naive(),
            sessions,
            network_health_score: 95.0,
        });
        assert_eq!(summary.total_online_minutes, 30);
        assert_eq!(summary.focus_minutes, 30);
    }

    #[test]
    fn network_focus_insight() {
        let sessions = vec![session(
            14,
            45,
            ActivityCategory::Work,
            Some(NetworkStability::Unstable),
        )];
        let insights = AnalyticsEngine::generate_insights(&sessions, &[], &[]);
        assert!(insights
            .iter()
            .any(|i| i.title.contains("Network hurt focus")));
        assert!(insights
            .iter()
            .any(|i| i.body.contains("14:00") && i.body.contains("Code")));
    }

    #[test]
    fn time_of_day_and_distraction() {
        let sessions = vec![
            session(9, 60, ActivityCategory::Work, None),
            session(14, 30, ActivityCategory::Distraction, None),
            session(14, 20, ActivityCategory::Distraction, None),
        ];
        let patterns = AnalyticsEngine::time_of_day_patterns(&sessions);
        assert!(patterns
            .iter()
            .any(|b| b.hour == 9 && b.productive_minutes == 60));
        assert!(patterns
            .iter()
            .any(|b| b.hour == 14 && b.distraction_minutes == 50));
        let impact = AnalyticsEngine::distraction_impact_pct(&sessions);
        assert!((impact - 45.45).abs() < 0.2);
    }

    #[test]
    fn monthly_summary_includes_patterns() {
        let sessions = vec![session(10, 40, ActivityCategory::Learning, None)];
        let monthly = AnalyticsEngine::monthly_summary(&sessions);
        assert_eq!(monthly.session_count, 1);
        assert!(!monthly.time_of_day.is_empty());
    }
}
