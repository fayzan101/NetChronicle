use std::net::SocketAddr;
use std::time::{Duration, Instant};

use async_trait::async_trait;
use chrono::Utc;
use netchronicle_common::NetworkStability;

use crate::{NetworkProbe, NetworkSample};

const DEFAULT_HOST: &str = "8.8.8.8:53";
const TIMEOUT: Duration = Duration::from_secs(3);

/// Measures reachability via TCP connect latency to a well-known host.
pub struct TcpProbe {
    target: SocketAddr,
}

impl Default for TcpProbe {
    fn default() -> Self {
        Self {
            target: DEFAULT_HOST.parse().expect("valid probe target"),
        }
    }
}

impl TcpProbe {
    pub fn new(target: impl Into<SocketAddr>) -> Self {
        Self {
            target: target.into(),
        }
    }
}

#[async_trait]
impl NetworkProbe for TcpProbe {
    async fn sample(&self) -> NetworkSample {
        let started = Instant::now();
        let result = tokio::time::timeout(TIMEOUT, tokio::net::TcpStream::connect(self.target)).await;

        match result {
            Ok(Ok(_)) => {
                let latency_ms = started.elapsed().as_secs_f32() * 1000.0;
                let stability = if latency_ms < 80.0 {
                    NetworkStability::Stable
                } else if latency_ms < 200.0 {
                    NetworkStability::Degraded
                } else {
                    NetworkStability::Unstable
                };

                NetworkSample {
                    latency_ms: Some(latency_ms),
                    packet_loss_pct: Some(0.0),
                    bandwidth_mbps: None,
                    stability,
                    disconnect: false,
                    recorded_at: Utc::now(),
                }
            }
            _ => NetworkSample {
                latency_ms: None,
                packet_loss_pct: Some(100.0),
                bandwidth_mbps: None,
                stability: NetworkStability::Offline,
                disconnect: true,
                recorded_at: Utc::now(),
            },
        }
    }
}
