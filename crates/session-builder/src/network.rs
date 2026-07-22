use chrono::{DateTime, Utc};
use netchronicle_common::NetworkStability;

use crate::NetworkObservation;

/// Aggregate network observations overlapping `[start, end]` into a session stability label.
///
/// Returns `None` when no samples fall in the window.
pub fn network_stability_for_window(
    observations: &[NetworkObservation],
    start: DateTime<Utc>,
    end: DateTime<Utc>,
) -> Option<NetworkStability> {
    let in_window: Vec<&NetworkObservation> = observations
        .iter()
        .filter(|obs| obs.recorded_at >= start && obs.recorded_at <= end)
        .collect();

    if in_window.is_empty() {
        return None;
    }

    if in_window.iter().any(|obs| obs.disconnect) {
        return Some(NetworkStability::Offline);
    }

    // Prefer metric-based reclassification when latency/loss were recorded.
    let latencies: Vec<f32> = in_window.iter().filter_map(|o| o.latency_ms).collect();
    let losses: Vec<f32> = in_window.iter().filter_map(|o| o.packet_loss_pct).collect();

    if !latencies.is_empty() || !losses.is_empty() {
        let avg_latency = if latencies.is_empty() {
            None
        } else {
            Some(latencies.iter().sum::<f32>() / latencies.len() as f32)
        };
        let avg_loss = if losses.is_empty() {
            None
        } else {
            Some(losses.iter().sum::<f32>() / losses.len() as f32)
        };
        return Some(netchronicle_network_monitor::classify_stability(
            avg_latency,
            avg_loss,
            false,
        ));
    }

    let unstable = in_window
        .iter()
        .filter(|obs| obs.stability == NetworkStability::Unstable)
        .count();
    if unstable * 2 >= in_window.len() {
        return Some(NetworkStability::Unstable);
    }

    let degraded = in_window
        .iter()
        .filter(|obs| obs.stability == NetworkStability::Degraded)
        .count();
    if degraded * 3 >= in_window.len() {
        return Some(NetworkStability::Degraded);
    }

    Some(NetworkStability::Stable)
}

#[cfg(test)]
mod tests {
    use super::*;
    use chrono::TimeZone;

    fn obs(
        at: DateTime<Utc>,
        stability: NetworkStability,
        disconnect: bool,
        latency_ms: Option<f32>,
        packet_loss_pct: Option<f32>,
    ) -> NetworkObservation {
        NetworkObservation {
            stability,
            disconnect,
            latency_ms,
            packet_loss_pct,
            recorded_at: at,
        }
    }

    #[test]
    fn empty_window_is_none() {
        let start = Utc.with_ymd_and_hms(2026, 1, 1, 9, 0, 0).unwrap();
        let end = start + chrono::Duration::hours(1);
        assert_eq!(network_stability_for_window(&[], start, end), None);
    }

    #[test]
    fn disconnect_marks_offline() {
        let start = Utc.with_ymd_and_hms(2026, 1, 1, 9, 0, 0).unwrap();
        let end = start + chrono::Duration::hours(1);
        let rows = vec![obs(
            start + chrono::Duration::minutes(10),
            NetworkStability::Stable,
            true,
            None,
            Some(100.0),
        )];
        assert_eq!(
            network_stability_for_window(&rows, start, end),
            Some(NetworkStability::Offline)
        );
    }

    #[test]
    fn avg_metrics_drive_classification() {
        let start = Utc.with_ymd_and_hms(2026, 1, 1, 9, 0, 0).unwrap();
        let end = start + chrono::Duration::hours(1);
        let rows = vec![
            obs(
                start + chrono::Duration::minutes(5),
                NetworkStability::Stable,
                false,
                Some(250.0),
                Some(0.0),
            ),
            obs(
                start + chrono::Duration::minutes(15),
                NetworkStability::Stable,
                false,
                Some(260.0),
                Some(0.0),
            ),
        ];
        assert_eq!(
            network_stability_for_window(&rows, start, end),
            Some(NetworkStability::Unstable)
        );
    }
}
