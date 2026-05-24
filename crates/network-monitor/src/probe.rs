use async_trait::async_trait;
use chrono::{DateTime, Utc};
use serde::{Deserialize, Serialize};

use netchronicle_common::NetworkStability;

#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct NetworkSample {
    pub latency_ms: Option<f32>,
    pub packet_loss_pct: Option<f32>,
    pub bandwidth_mbps: Option<f32>,
    pub stability: NetworkStability,
    pub disconnect: bool,
    pub recorded_at: DateTime<Utc>,
}

#[async_trait]
pub trait NetworkProbe: Send + Sync {
    async fn sample(&self) -> NetworkSample;
}
