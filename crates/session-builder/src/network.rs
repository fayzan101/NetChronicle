use chrono::{DateTime, Utc};
use netchronicle_common::NetworkStability;

use crate::NetworkObservation;

pub fn network_stability_for_window(
    observations: &[NetworkObservation],
    start: DateTime<Utc>,
    end: DateTime<Utc>,
) -> NetworkStability {
    let in_window: Vec<&NetworkObservation> = observations
        .iter()
        .filter(|obs| obs.recorded_at >= start && obs.recorded_at <= end)
        .collect();

    if in_window.is_empty() {
        return NetworkStability::Stable;
    }

    if in_window.iter().any(|obs| obs.disconnect) {
        return NetworkStability::Offline;
    }

    let unstable = in_window
        .iter()
        .filter(|obs| obs.stability == NetworkStability::Unstable)
        .count();
    if unstable * 2 >= in_window.len() {
        return NetworkStability::Unstable;
    }

    let degraded = in_window
        .iter()
        .filter(|obs| obs.stability == NetworkStability::Degraded)
        .count();
    if degraded * 3 >= in_window.len() {
        return NetworkStability::Degraded;
    }

    NetworkStability::Stable
}
