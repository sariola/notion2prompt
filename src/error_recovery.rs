// src/error_recovery.rs
//! Retry with exponential backoff for API operations.

use crate::error::AppError;
use std::time::Duration;

/// Retries an async operation with exponential backoff.
pub async fn retry_with_backoff<F, T, Fut>(
    mut operation: F,
    max_attempts: u32,
    initial_delay: Duration,
    max_delay: Duration,
) -> Result<T, AppError>
where
    F: FnMut() -> Fut,
    Fut: std::future::Future<Output = Result<T, AppError>>,
{
    let mut delay = initial_delay;
    let mut last_error = None;

    for attempt in 1..=max_attempts {
        match operation().await {
            Ok(result) => return Ok(result),
            Err(e) => {
                last_error = Some(e);

                if attempt < max_attempts {
                    log::warn!("Attempt {} failed, retrying after {:?}", attempt, delay);
                    tokio::time::sleep(delay).await;

                    // Exponential backoff with cap
                    delay = std::cmp::min(delay * 2, max_delay);
                }
            }
        }
    }

    Err(last_error.unwrap_or_else(|| AppError::InternalError {
        message: "Retry failed with no error".to_string(),
        source: None,
    }))
}
