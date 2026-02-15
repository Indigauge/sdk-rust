// Retry support module for HTTP requests
//
// This module provides a simple retry mechanism that works with bevy_mod_reqwest.
// Since bevy_mod_reqwest doesn't support middleware directly, we configure the
// underlying reqwest client with connection pooling, TCP keep-alive, and other
// settings that improve reliability at the network level.
//
// For application-level retries (e.g., retrying on 5xx errors), users can implement
// custom logic in their error handlers using the retry configuration provided in
// IndigaugeConfig (max_retries and retry_delay).

use std::time::Duration;

/// Maximum backoff delay in seconds to prevent unreasonably long waits
const MAX_BACKOFF_SECS: f64 = 60.0;

/// Calculate exponential backoff delay in seconds with a maximum cap
///
/// Uses exponential backoff (base_delay * 2^attempt) but caps the result
/// to prevent overflow and unreasonably long delays.
///
/// # Arguments
/// * `attempt` - The retry attempt number (0-indexed)
/// * `base_delay_secs` - The base delay in seconds
///
/// # Returns
/// The calculated backoff delay, capped at MAX_BACKOFF_SECS
pub fn calculate_backoff_delay(attempt: u32, base_delay_secs: f64) -> f64 {
  // Cap the exponent to prevent overflow
  let capped_attempt = attempt.min(10); // 2^10 = 1024, reasonable upper bound
  let delay = base_delay_secs * 2_f64.powi(capped_attempt as i32);
  delay.min(MAX_BACKOFF_SECS)
}

/// Check if a request should be retried based on attempt count
///
/// # Arguments
/// * `attempt` - The retry attempt number (0-indexed)
/// * `max_retries` - Maximum number of retries allowed
///
/// # Returns
/// `true` if the request should be retried, `false` otherwise
pub fn should_retry(attempt: u32, max_retries: u32) -> bool {
  attempt < max_retries
}

#[cfg(test)]
mod tests {
  use super::*;

  #[test]
  fn test_calculate_backoff_delay() {
    // Test basic exponential backoff
    assert_eq!(calculate_backoff_delay(0, 1.0), 1.0);
    assert_eq!(calculate_backoff_delay(1, 1.0), 2.0);
    assert_eq!(calculate_backoff_delay(2, 1.0), 4.0);
    assert_eq!(calculate_backoff_delay(3, 1.0), 8.0);

    // Test with different base delay
    assert_eq!(calculate_backoff_delay(0, 0.5), 0.5);
    assert_eq!(calculate_backoff_delay(1, 0.5), 1.0);

    // Test capping at MAX_BACKOFF_SECS
    let large_attempt_delay = calculate_backoff_delay(20, 1.0);
    assert!(large_attempt_delay <= MAX_BACKOFF_SECS);

    // Test that very large attempts don't panic or overflow
    let very_large_delay = calculate_backoff_delay(u32::MAX, 1.0);
    assert!(very_large_delay <= MAX_BACKOFF_SECS);
  }

  #[test]
  fn test_should_retry() {
    assert!(should_retry(0, 3));
    assert!(should_retry(1, 3));
    assert!(should_retry(2, 3));
    assert!(!should_retry(3, 3));
    assert!(!should_retry(4, 3));

    // Test edge cases
    assert!(!should_retry(0, 0));
    assert!(should_retry(0, 1));
  }
}

