use std::net::SocketAddr;
use std::time::{Duration, Instant};

use async_trait::async_trait;
use chrono::Utc;

use crate::classify::classify_stability;
use crate::{NetworkProbe, NetworkSample};

const DEFAULT_ADDR: &str = "8.8.8.8:53";
const TIMEOUT: Duration = Duration::from_secs(3);

/// TCP connect latency fallback when ICMP is unavailable.
pub struct TcpProbe {
    target: SocketAddr,
}

impl Default for TcpProbe {
    fn default() -> Self {
        Self {
            target: DEFAULT_ADDR.parse().expect("valid probe target"),
        }
    }
}

impl TcpProbe {
    pub fn new(target: impl Into<SocketAddr>) -> Self {
        Self {
            target: target.into(),
        }
    }

    pub fn from_host_port(host: &str, port: u16) -> Option<Self> {
        let addr: SocketAddr = format!("{host}:{port}").parse().ok()?;
        Some(Self::new(addr))
    }
}

#[async_trait]
impl NetworkProbe for TcpProbe {
    async fn sample(&self) -> NetworkSample {
        let started = Instant::now();
        let result =
            tokio::time::timeout(TIMEOUT, tokio::net::TcpStream::connect(self.target)).await;

        match result {
            Ok(Ok(_)) => {
                let latency_ms = started.elapsed().as_secs_f32() * 1000.0;
                let packet_loss_pct = Some(0.0);
                NetworkSample {
                    latency_ms: Some(latency_ms),
                    packet_loss_pct,
                    bandwidth_mbps: None,
                    stability: classify_stability(Some(latency_ms), packet_loss_pct, false),
                    disconnect: false,
                    recorded_at: Utc::now(),
                }
            }
            _ => NetworkSample {
                latency_ms: None,
                packet_loss_pct: Some(100.0),
                bandwidth_mbps: None,
                stability: classify_stability(None, Some(100.0), true),
                disconnect: true,
                recorded_at: Utc::now(),
            },
        }
    }
}
