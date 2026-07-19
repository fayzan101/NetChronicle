use netchronicle_common::NetworkStability;

/// Classify network quality from measured metrics.
///
/// Precedence: disconnect → offline; then the worse of latency vs packet-loss bands.
pub fn classify_stability(
    latency_ms: Option<f32>,
    packet_loss_pct: Option<f32>,
    disconnect: bool,
) -> NetworkStability {
    if disconnect {
        return NetworkStability::Offline;
    }

    let from_loss = match packet_loss_pct {
        Some(loss) if loss >= 50.0 => NetworkStability::Unstable,
        Some(loss) if loss >= 15.0 => NetworkStability::Degraded,
        Some(loss) if loss > 0.0 => NetworkStability::Degraded,
        _ => NetworkStability::Stable,
    };

    let from_latency = match latency_ms {
        Some(ms) if ms < 80.0 => NetworkStability::Stable,
        Some(ms) if ms < 200.0 => NetworkStability::Degraded,
        Some(_) => NetworkStability::Unstable,
        None => NetworkStability::Unstable,
    };

    worse(from_loss, from_latency)
}

fn worse(a: NetworkStability, b: NetworkStability) -> NetworkStability {
    use NetworkStability::*;
    let rank = |s: NetworkStability| match s {
        Stable => 0,
        Degraded => 1,
        Unstable => 2,
        Offline => 3,
    };
    if rank(a) >= rank(b) {
        a
    } else {
        b
    }
}

/// True when latency or loss looks like a spike worth surfacing as an event.
pub fn is_spike(latency_ms: Option<f32>, packet_loss_pct: Option<f32>) -> bool {
    matches!(latency_ms, Some(ms) if ms >= 200.0)
        || matches!(packet_loss_pct, Some(loss) if loss >= 15.0)
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn disconnect_is_offline() {
        assert_eq!(
            classify_stability(Some(10.0), Some(0.0), true),
            NetworkStability::Offline
        );
    }

    #[test]
    fn low_latency_no_loss_is_stable() {
        assert_eq!(
            classify_stability(Some(40.0), Some(0.0), false),
            NetworkStability::Stable
        );
    }

    #[test]
    fn high_latency_is_unstable() {
        assert_eq!(
            classify_stability(Some(250.0), Some(0.0), false),
            NetworkStability::Unstable
        );
    }

    #[test]
    fn moderate_loss_degrades() {
        assert_eq!(
            classify_stability(Some(40.0), Some(20.0), false),
            NetworkStability::Degraded
        );
    }

    #[test]
    fn loss_outranks_good_latency() {
        assert_eq!(
            classify_stability(Some(30.0), Some(60.0), false),
            NetworkStability::Unstable
        );
    }

    #[test]
    fn spike_detection() {
        assert!(is_spike(Some(220.0), Some(0.0)));
        assert!(is_spike(Some(50.0), Some(20.0)));
        assert!(!is_spike(Some(50.0), Some(0.0)));
    }
}
