use std::time::Duration;
use std::thread;
use crate::errors::SyncError;

/// Retry configuration
#[derive(Debug, Clone)]
pub struct RetryConfig {
    pub max_attempts: u32,
    pub initial_delay_ms: u64,
    pub max_delay_ms: u64,
    pub backoff_multiplier: f64,
}

impl Default for RetryConfig {
    fn default() -> Self {
        Self {
            max_attempts: 3,
            initial_delay_ms: 100,
            max_delay_ms: 5000,
            backoff_multiplier: 2.0,
        }
    }
}

/// Retry result with attempt information
#[derive(Debug, Clone)]
pub struct RetryResult<T> {
    pub result: Result<T, SyncError>,
    pub attempts: u32,
    pub total_duration_ms: u64,
}

/// Retry a function with exponential backoff
pub fn retry_with_backoff<F, T>(
    func: F,
    config: &RetryConfig,
    operation_name: &str,
) -> RetryResult<T>
where
    F: Fn() -> Result<T, SyncError>,
{
    let mut last_error = None;
    let mut total_duration = 0u64;

    for attempt in 1..=config.max_attempts {
        let start = std::time::Instant::now();

        match func() {
            Ok(value) => {
                if attempt > 1 {
                eprintln!(
                    "Operation '{}' succeeded after {} attempt(s)",
                    operation_name,
                    attempt
                );
                }
                return RetryResult {
                    result: Ok(value),
                    attempts: attempt,
                    total_duration_ms: total_duration,
                };
            }
            Err(e) => {
                let elapsed = start.elapsed().as_millis() as u64;
                total_duration += elapsed;

                last_error = Some(e.clone());

                // Check if error is retryable
                if !e.is_retryable() {
                eprintln!(
                    "WARNING: Operation '{}' failed with non-retryable error: {}",
                    operation_name,
                    e.message()
                );
                    return RetryResult {
                        result: Err(e),
                        attempts: attempt,
                        total_duration_ms: total_duration,
                    };
                }

                // If this is the last attempt, return the error
                if attempt >= config.max_attempts {
                eprintln!(
                    "ERROR: Operation '{}' failed after {} attempts: {}",
                    operation_name,
                    attempt,
                    e.message()
                );
                    return RetryResult {
                        result: Err(e),
                        attempts: attempt,
                        total_duration_ms: total_duration,
                    };
                }

                // Calculate delay with exponential backoff
                let delay_ms = calculate_delay(attempt, config);
                eprintln!(
                    "WARNING: Operation '{}' failed on attempt {}/{}: {}. Retrying in {}ms...",
                    operation_name,
                    attempt,
                    config.max_attempts,
                    e.message(),
                    delay_ms
                );

                thread::sleep(Duration::from_millis(delay_ms));
                total_duration += delay_ms;
            }
        }
    }

    // Should not reach here, but handle it gracefully
    RetryResult {
        result: Err(last_error.unwrap_or_else(|| {
            SyncError::Custom(format!("Operation '{}' failed after {} attempts", operation_name, config.max_attempts))
        })),
        attempts: config.max_attempts,
        total_duration_ms: total_duration,
    }
}

/// Calculate exponential backoff delay
fn calculate_delay(attempt: u32, config: &RetryConfig) -> u64 {
    let delay = (config.initial_delay_ms as f64) * (config.backoff_multiplier.powi((attempt as i32) - 2)).max(1.0);
    delay.min(config.max_delay_ms as f64) as u64
}

#[cfg(test)]
mod tests {
    use super::*;
    use crate::errors::SyncError;
    use std::cell::RefCell;

    #[test]
    fn test_retry_success_first_attempt() {
        let config = RetryConfig::default();
        let call_count = RefCell::new(0);

        let result: RetryResult<&str> = retry_with_backoff(
            || {
                *call_count.borrow_mut() += 1;
                Ok("success")
            },
            &config,
            "test_operation",
        );

        assert!(result.result.is_ok());
        assert_eq!(result.attempts, 1);
        assert_eq!(*call_count.borrow(), 1);
    }

    #[test]
    fn test_retry_success_after_retries() {
        let config = RetryConfig {
            max_attempts: 3,
            initial_delay_ms: 10,
            max_delay_ms: 100,
            backoff_multiplier: 2.0,
        };
        let call_count = RefCell::new(0);

        let result: RetryResult<&str> = retry_with_backoff(
            || {
                *call_count.borrow_mut() += 1;
                if *call_count.borrow() < 2 {
                    Err(SyncError::NetworkError("Temporary failure".to_string()))
                } else {
                    Ok("success")
                }
            },
            &config,
            "test_operation",
        );

        assert!(result.result.is_ok());
        assert_eq!(result.attempts, 2);
        assert_eq!(*call_count.borrow(), 2);
    }

    #[test]
    fn test_retry_non_retryable_error() {
        let config = RetryConfig::default();
        let call_count = RefCell::new(0);

        let result: RetryResult<&str> = retry_with_backoff(
            || {
                *call_count.borrow_mut() += 1;
                Err(SyncError::CorruptedFile("File is corrupted".to_string()))
            },
            &config,
            "test_operation",
        );

        assert!(result.result.is_err());
        assert_eq!(result.attempts, 1);
        assert_eq!(*call_count.borrow(), 1);
    }

    #[test]
    fn test_retry_max_attempts_exceeded() {
        let config = RetryConfig {
            max_attempts: 2,
            initial_delay_ms: 10,
            max_delay_ms: 100,
            backoff_multiplier: 2.0,
        };
        let call_count = RefCell::new(0);

        let result: RetryResult<&str> = retry_with_backoff(
            || {
                *call_count.borrow_mut() += 1;
                Err(SyncError::NetworkError("Persistent failure".to_string()))
            },
            &config,
            "test_operation",
        );

        assert!(result.result.is_err());
        assert_eq!(result.attempts, 2);
        assert_eq!(*call_count.borrow(), 2);
    }

    #[test]
    fn test_calculate_delay() {
        let config = RetryConfig {
            initial_delay_ms: 100,
            max_delay_ms: 5000,
            backoff_multiplier: 2.0,
            max_attempts: 5,
        };

        // First retry: initial delay
        assert_eq!(calculate_delay(1, &config), 100);

        // Second retry: initial delay * 2^0 = 100ms
        assert_eq!(calculate_delay(2, &config), 100);

        // Third retry: initial delay * 2^1 = 200ms
        assert_eq!(calculate_delay(3, &config), 200);

        // Fourth retry: initial delay * 2^2 = 400ms
        assert_eq!(calculate_delay(4, &config), 400);

        // Should cap at max_delay
        let config_max = RetryConfig {
            initial_delay_ms: 100,
            max_delay_ms: 200,
            backoff_multiplier: 2.0,
            max_attempts: 5,
        };
        assert_eq!(calculate_delay(5, &config_max), 200);
    }
}

