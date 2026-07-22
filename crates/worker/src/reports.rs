use chrono::{Datelike, NaiveDate};
use netchronicle_analytics::{AnalyticsEngine, DailyAnalyticsInput};
use netchronicle_db::{
    session_row_to_common, AnalyticsRepository, DbPool, NetworkRepository, ReportRepository,
    SessionRepository,
};
use tracing::info;
use uuid::Uuid;

fn day_bounds(day: NaiveDate) -> (chrono::DateTime<chrono::Utc>, chrono::DateTime<chrono::Utc>) {
    let start = day.and_hms_opt(0, 0, 0).unwrap().and_utc();
    let end = (day + chrono::Duration::days(1))
        .and_hms_opt(0, 0, 0)
        .unwrap()
        .and_utc();
    (start, end)
}

fn week_bounds(day: NaiveDate) -> (NaiveDate, NaiveDate) {
    let weekday = day.weekday().num_days_from_monday() as i64;
    let week_start = day - chrono::Duration::days(weekday);
    let week_end = week_start + chrono::Duration::days(6);
    (week_start, week_end)
}

fn month_bounds(day: NaiveDate) -> (NaiveDate, NaiveDate) {
    let month_start = NaiveDate::from_ymd_opt(day.year(), day.month(), 1).unwrap();
    let month_end = if day.month() == 12 {
        NaiveDate::from_ymd_opt(day.year() + 1, 1, 1).unwrap() - chrono::Duration::days(1)
    } else {
        NaiveDate::from_ymd_opt(day.year(), day.month() + 1, 1).unwrap() - chrono::Duration::days(1)
    };
    (month_start, month_end)
}

pub async fn compute_daily_report(
    user_id: Uuid,
    pool: &DbPool,
    day: NaiveDate,
) -> anyhow::Result<()> {
    let (from, to) = day_bounds(day);
    let sessions: Vec<_> = SessionRepository::new(pool)
        .list(user_id, from, to, 1000, 0)
        .await?
        .into_iter()
        .map(session_row_to_common)
        .collect();

    let network_score = NetworkRepository::new(pool)
        .stability_score(user_id, from)
        .await?;

    let summary = AnalyticsEngine::daily_summary(&DailyAnalyticsInput {
        date: day,
        sessions: sessions.clone(),
        network_health_score: network_score,
    });

    let patterns = AnalyticsEngine::time_of_day_patterns(&sessions);
    let distraction_impact = AnalyticsEngine::distraction_impact_pct(&sessions);

    let payload = serde_json::json!({
        "productivityScore": summary.productivity_score,
        "totalOnlineMinutes": summary.total_online_minutes,
        "networkHealthScore": summary.network_health_score,
        "distractionRatio": summary.distraction_ratio,
        "distractionImpactPct": distraction_impact,
        "focusMinutes": summary.focus_minutes,
        "timeOfDay": patterns,
    });

    ReportRepository::new(pool)
        .upsert(user_id, "daily", day, day, payload)
        .await?;

    Ok(())
}

pub async fn compute_period_report(
    user_id: Uuid,
    pool: &DbPool,
    report_type: &str,
    period_start: NaiveDate,
    period_end: NaiveDate,
) -> anyhow::Result<()> {
    let (from, _) = day_bounds(period_start);
    let (_, to) = day_bounds(period_end);

    let sessions: Vec<_> = SessionRepository::new(pool)
        .list(user_id, from, to, 10_000, 0)
        .await?
        .into_iter()
        .map(session_row_to_common)
        .collect();

    let period = if report_type == "monthly" {
        AnalyticsEngine::monthly_summary(&sessions)
    } else {
        let weekly = AnalyticsEngine::weekly_summary(&sessions);
        netchronicle_analytics::PeriodSummary {
            total_online_minutes: weekly.total_online_minutes,
            productive_minutes: weekly.productive_minutes,
            session_count: weekly.session_count,
            average_productivity_score: weekly.average_productivity_score,
            category_minutes: weekly.category_minutes,
            distraction_impact_pct: AnalyticsEngine::distraction_impact_pct(&sessions),
            time_of_day: AnalyticsEngine::time_of_day_patterns(&sessions),
        }
    };

    let analytics = AnalyticsRepository::new(pool);
    let top_apps = analytics.top_apps(user_id, from, to, 10).await?;
    let top_domains = analytics.top_domains(user_id, from, to, 10).await?;

    let summary = serde_json::json!({
        "totalOnlineMinutes": period.total_online_minutes,
        "productiveMinutes": period.productive_minutes,
        "sessionCount": period.session_count,
        "averageProductivityScore": period.average_productivity_score,
        "distractionImpactPct": period.distraction_impact_pct,
        "categoryMinutes": period.category_minutes,
        "timeOfDay": period.time_of_day,
        "topApps": top_apps.into_iter().map(|(name, secs)| serde_json::json!({"app": name, "minutes": secs / 60})).collect::<Vec<_>>(),
        "topDomains": top_domains.into_iter().map(|(domain, secs)| serde_json::json!({"domain": domain, "minutes": secs / 60})).collect::<Vec<_>>(),
    });

    ReportRepository::new(pool)
        .upsert(user_id, report_type, period_start, period_end, summary)
        .await?;

    Ok(())
}

pub async fn compute_reports_for_user(
    user_id: Uuid,
    pool: &DbPool,
    today: NaiveDate,
    lookback_days: i64,
) -> anyhow::Result<usize> {
    let mut count = 0usize;

    for offset in 0..lookback_days {
        let day = today - chrono::Duration::days(offset);
        compute_daily_report(user_id, pool, day).await?;
        count += 1;
    }

    // Weekly reports for each week intersecting lookback
    let mut week_starts = std::collections::BTreeSet::new();
    for offset in 0..lookback_days {
        let day = today - chrono::Duration::days(offset);
        let (week_start, _) = week_bounds(day);
        week_starts.insert(week_start);
    }
    for week_start in week_starts {
        let week_end = week_start + chrono::Duration::days(6);
        compute_period_report(user_id, pool, "weekly", week_start, week_end).await?;
        count += 1;
    }

    // Monthly reports for months intersecting lookback
    let mut month_starts = std::collections::BTreeSet::new();
    for offset in 0..lookback_days {
        let day = today - chrono::Duration::days(offset);
        let (month_start, _) = month_bounds(day);
        month_starts.insert(month_start);
    }
    for month_start in month_starts {
        let (_, month_end) = month_bounds(month_start);
        compute_period_report(user_id, pool, "monthly", month_start, month_end).await?;
        count += 1;
    }

    info!(%user_id, count, "reports cached");
    Ok(count)
}

#[cfg(test)]
mod tests {
    use super::*;
    use chrono::Weekday;

    #[test]
    fn week_bounds_monday_start() {
        // 2026-07-22 is Wednesday
        let day = NaiveDate::from_ymd_opt(2026, 7, 22).unwrap();
        let (start, end) = week_bounds(day);
        assert_eq!(start.weekday(), Weekday::Mon);
        assert_eq!(end.weekday(), Weekday::Sun);
        assert_eq!(start, NaiveDate::from_ymd_opt(2026, 7, 20).unwrap());
    }

    #[test]
    fn month_bounds_july() {
        let day = NaiveDate::from_ymd_opt(2026, 7, 22).unwrap();
        let (start, end) = month_bounds(day);
        assert_eq!(start, NaiveDate::from_ymd_opt(2026, 7, 1).unwrap());
        assert_eq!(end, NaiveDate::from_ymd_opt(2026, 7, 31).unwrap());
    }
}
