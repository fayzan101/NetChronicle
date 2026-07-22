use std::time::Duration;

use async_trait::async_trait;
use chrono::Utc;

use crate::bandwidth::estimate_bandwidth_mbps;
use crate::classify::classify_stability;
use crate::connectivity::has_basic_connectivity;
use crate::icmp::ping_host;
use crate::tcp::TcpProbe;
use crate::{NetworkProbe, NetworkSample};

/// Configuration for the default composite network probe.
#[derive(Debug, Clone)]
pub struct ProbeConfig {
    pub host: String,
    pub tcp_port: u16,
    pub ping_count: u32,
    pub ping_timeout: Duration,
    pub bandwidth_enabled: bool,
    pub bandwidth_url: String,
    pub bandwidth_bytes: usize,
}

impl Default for ProbeConfig {
    fn default() -> Self {
        Self {
            host: "8.8.8.8".into(),
            tcp_port: 53,
            ping_count: 4,
            ping_timeout: Duration::from_secs(1),
            bandwidth_enabled: false,
            bandwidth_url: crate::bandwidth::default_bandwidth_url().into(),
            bandwidth_bytes: 100_000,
        }
    }
}

impl ProbeConfig {
    pub fn from_env() -> Self {
        let mut cfg = Self::default();

        if let Ok(host) = std::env::var("NETWORK_PROBE_HOST") {
            if !host.trim().is_empty() {
                cfg.host = host.trim().to_string();
            }
        }
        if let Ok(port) = std::env::var("NETWORK_PROBE_TCP_PORT") {
            if let Ok(p) = port.parse() {
                cfg.tcp_port = p;
            }
        }
        if let Ok(count) = std::env::var("NETWORK_PING_COUNT") {
            if let Ok(c) = count.parse() {
                cfg.ping_count = c;
            }
        }
        if let Ok(enabled) = std::env::var("NETWORK_BANDWIDTH_ENABLED") {
            cfg.bandwidth_enabled = matches!(
                enabled.to_ascii_lowercase().as_str(),
                "1" | "true" | "yes" | "on"
            );
        }
        if let Ok(url) = std::env::var("NETWORK_BANDWIDTH_URL") {
            if !url.trim().is_empty() {
                cfg.bandwidth_url = url;
            }
        }
        if let Ok(bytes) = std::env::var("NETWORK_BANDWIDTH_BYTES") {
            if let Ok(b) = bytes.parse() {
                cfg.bandwidth_bytes = b;
            }
        }

        cfg
    }
}

/// ICMP ping with TCP fallback, optional bandwidth, and disconnect detection.
pub struct CompositeProbe {
    config: ProbeConfig,
    tcp: TcpProbe,
}

impl Default for CompositeProbe {
    fn default() -> Self {
        Self::new(ProbeConfig::default())
    }
}

impl CompositeProbe {
    pub fn new(config: ProbeConfig) -> Self {
        let tcp = TcpProbe::from_host_port(&config.host, config.tcp_port).unwrap_or_default();
        Self { config, tcp }
    }

    pub fn from_env() -> Self {
        Self::new(ProbeConfig::from_env())
    }
}

#[async_trait]
impl NetworkProbe for CompositeProbe {
    async fn sample(&self) -> NetworkSample {
        let online = has_basic_connectivity(&self.config.host, self.config.tcp_port).await;

        let icmp = ping_host(
            &self.config.host,
            self.config.ping_count,
            self.config.ping_timeout,
        )
        .await;

        let (latency_ms, packet_loss_pct, used_tcp_fallback) = if let Some(icmp) = icmp {
            // Total failure of the ping batch → treat as disconnect candidate.
            if icmp.received == 0 {
                (None, Some(100.0), false)
            } else {
                (icmp.latency_ms, Some(icmp.packet_loss_pct), false)
            }
        } else {
            // ICMP unavailable (no ping binary / blocked) — fall back to TCP connect.
            let tcp_sample = self.tcp.sample().await;
            (tcp_sample.latency_ms, tcp_sample.packet_loss_pct, true)
        };

        let disconnect = !online
            || matches!(packet_loss_pct, Some(loss) if loss >= 100.0)
            || (latency_ms.is_none() && !used_tcp_fallback);

        // If we still look connected via TCP fallback but ICMP failed entirely, trust TCP.
        let disconnect = if used_tcp_fallback && latency_ms.is_some() {
            false
        } else {
            disconnect
        };

        let bandwidth_mbps = if self.config.bandwidth_enabled && !disconnect {
            estimate_bandwidth_mbps(&self.config.bandwidth_url, self.config.bandwidth_bytes).await
        } else {
            None
        };

        let stability = classify_stability(latency_ms, packet_loss_pct, disconnect);

        NetworkSample {
            latency_ms,
            packet_loss_pct,
            bandwidth_mbps,
            stability,
            disconnect,
            recorded_at: Utc::now(),
        }
    }
}
