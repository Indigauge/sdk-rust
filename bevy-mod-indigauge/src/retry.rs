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

use indigauge_types::prelude::IndigaugeConfig;

/// Calculate exponential backoff delay in seconds
pub fn calculate_backoff_delay(attempt: u32, base_delay_secs: f64) -> f64 {
  base_delay_secs * 2_f64.powi(attempt as i32)
}

/// Check if a request should be retried based on attempt count
pub fn should_retry(attempt: u32, config: &IndigaugeConfig) -> bool {
  attempt < config.max_retries()
}
