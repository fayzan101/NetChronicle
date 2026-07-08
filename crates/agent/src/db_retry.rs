use std::future::Future;
use std::time::Duration;
use tracing::warn;

pub async fn with_db_retry<T, F, Fut>(mut operation: F) -> anyhow::Result<T>
where
    F: FnMut() -> Fut,
    Fut: Future<Output = anyhow::Result<T>>,
{
    const MAX_ATTEMPTS: u32 = 3;
    let mut attempt = 0;

    loop {
        attempt += 1;
        match operation().await {
            Ok(value) => return Ok(value),
            Err(error) if attempt < MAX_ATTEMPTS && is_transient(&error) => {
                warn!(%error, attempt, "database operation failed, retrying");
                tokio::time::sleep(Duration::from_secs(attempt as u64)).await;
            }
            Err(error) => return Err(error),
        }
    }
}

fn is_transient(error: &anyhow::Error) -> bool {
    let message = error.to_string().to_lowercase();
    message.contains("connection")
        || message.contains("timeout")
        || message.contains("broken pipe")
        || message.contains("pool timed out")
}
