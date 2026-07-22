use std::time::Duration;

use tracing::info;

use crate::{NetworkProbe, NetworkSample};

/// Periodic network sampling loop (invoked by the agent).
pub struct NetworkSampler<P: NetworkProbe> {
    probe: P,
    interval: Duration,
}

impl<P: NetworkProbe> NetworkSampler<P> {
    pub fn new(probe: P, interval: Duration) -> Self {
        Self { probe, interval }
    }

    pub async fn run<F>(&self, mut on_sample: F)
    where
        F: FnMut(NetworkSample) + Send,
    {
        loop {
            let sample = self.probe.sample().await;
            on_sample(sample);
            tokio::time::sleep(self.interval).await;
        }
    }

    pub fn interval(&self) -> Duration {
        self.interval
    }
}

impl<P: NetworkProbe> NetworkSampler<P> {
    pub fn log_config(&self) {
        info!(
            interval_secs = self.interval.as_secs(),
            "network sampler configured"
        );
    }
}
