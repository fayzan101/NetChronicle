use chrono::NaiveDate;
use netchronicle_common::AppActivityEvent;
use netchronicle_db::{
    parse_category, parse_stability, ActivityRepository, DbPool, NetworkRepository,
    SessionRepository,
};
use netchronicle_session_builder::{
    NetworkObservation, SessionBuilder, SessionBuilderConfig, TrackedAppLog,
};
use tracing::{debug, info};
use uuid::Uuid;

pub fn rebuild_days_from(today: NaiveDate, lookback_days: i64) -> Vec<NaiveDate> {
    (0..lookback_days)
        .map(|offset| today - chrono::Duration::days(offset))
        .collect()
}

pub async fn rebuild_sessions_for_day(
    user_id: Uuid,
    pool: &DbPool,
    day: NaiveDate,
) -> anyhow::Result<usize> {
    let start = day.and_hms_opt(0, 0, 0).unwrap().and_utc();
    let end = (day + chrono::Duration::days(1))
        .and_hms_opt(0, 0, 0)
        .unwrap()
        .and_utc();

    let activity = ActivityRepository::new(pool);
    let network = NetworkRepository::new(pool);
    let sessions_repo = SessionRepository::new(pool);

    sessions_repo.clear_for_day(user_id, day).await?;

    let app_rows = activity
        .list_app_logs(user_id, start, end, 10_000, 0)
        .await?;
    if app_rows.is_empty() {
        debug!(%day, "no app logs to build sessions from");
        return Ok(0);
    }

    let network_rows = network.list_since(user_id, start, 10_000).await?;
    let network: Vec<NetworkObservation> = network_rows
        .into_iter()
        .filter(|row| row.recorded_at < end)
        .map(|row| NetworkObservation {
            stability: row
                .stability
                .as_deref()
                .map(parse_stability)
                .unwrap_or(netchronicle_common::NetworkStability::Stable),
            disconnect: row.disconnect,
            latency_ms: row.latency_ms,
            packet_loss_pct: row.packet_loss_pct,
            recorded_at: row.recorded_at,
        })
        .collect();

    let tracked: Vec<TrackedAppLog> = app_rows
        .into_iter()
        .map(|row| TrackedAppLog {
            log_id: row.id,
            event: AppActivityEvent {
                app_name: row.app_name,
                window_title: row.window_title,
                duration_sec: row.duration_sec as u32,
                category: parse_category(&row.category),
                recorded_at: row.recorded_at,
            },
        })
        .collect();

    let builder = SessionBuilder::new(
        user_id,
        SessionBuilderConfig {
            idle_gap: std::time::Duration::from_secs(
                std::env::var("SESSION_IDLE_GAP_SECS")
                    .ok()
                    .and_then(|v| v.parse().ok())
                    .unwrap_or(300),
            ),
            min_session_duration: std::time::Duration::from_secs(
                std::env::var("SESSION_MIN_DURATION_SECS")
                    .ok()
                    .and_then(|v| v.parse().ok())
                    .unwrap_or(60),
            ),
        },
    );

    let built = builder.build_from_logs(&tracked, &network);
    let mut count = 0usize;

    for session in built {
        let session_id = sessions_repo.insert(&session.draft).await?;
        sessions_repo
            .link_app_logs(session_id, &session.log_ids)
            .await?;

        if let Some(end_time) = session.draft.end_time {
            let linked = sessions_repo
                .link_website_logs_in_window(
                    session_id,
                    user_id,
                    session.draft.start_time,
                    end_time,
                )
                .await?;
            debug!(%session_id, linked, "linked website logs to session");
        }

        count += 1;
    }

    info!(%day, count, "rebuilt sessions");
    Ok(count)
}

pub async fn rebuild_sessions_for_lookback(
    user_id: Uuid,
    pool: &DbPool,
    today: NaiveDate,
    lookback_days: i64,
) -> anyhow::Result<usize> {
    let days = rebuild_days_from(today, lookback_days);
    let mut total = 0usize;

    for day in days.into_iter().rev() {
        total += rebuild_sessions_for_day(user_id, pool, day).await?;
    }

    info!(lookback_days, total, "session rebuild lookback complete");
    Ok(total)
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn lookback_days_ordered_newest_first() {
        let today = NaiveDate::from_ymd_opt(2026, 7, 22).unwrap();
        let days = rebuild_days_from(today, 3);
        assert_eq!(days.len(), 3);
        assert_eq!(days[0], today);
        assert_eq!(days[2], today - chrono::Duration::days(2));
    }
}
