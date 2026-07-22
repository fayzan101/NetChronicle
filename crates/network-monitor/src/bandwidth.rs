use std::time::{Duration, Instant};
use tracing::debug;
const DEFAULT_URL: &str = "https://speed.cloudflare.com/__down?bytes=100000";
const TIMEOUT: Duration = Duration::from_secs(8);
pub async fn estimate_bandwidth_mbps(url: &str, max_bytes: usize) -> Option<f32> {
    let url = if url.is_empty() { DEFAULT_URL } else { url };
    let max_bytes = max_bytes.clamp(10_000, 2_000_000);
    let client = reqwest::Client::builder().timeout(TIMEOUT).build().ok()?;
    let started = Instant::now();
    let response = client.get(url).send().await.ok()?;
    if !response.status().is_success() {
        debug!(status = %response.status(), "bandwidth probe non-success");
        return None;
    }
    let mut downloaded = 0usize;
    let mut stream = response.bytes_stream();
    use futures_util::StreamExt;
    while let Some(chunk) = stream.next().await {
        let chunk = chunk.ok()?;
        downloaded += chunk.len();
        if downloaded >= max_bytes {
            break;
        }
    }
    let elapsed = started.elapsed().as_secs_f32();
    if elapsed <= 0.0 || downloaded == 0 {
        return None;
    }
    let mbps = (downloaded as f32 * 8.0) / (elapsed * 1_000_000.0);
    Some(mbps)
}
pub fn default_bandwidth_url() -> &'static str {
    DEFAULT_URL
}
