use std::{env, time::Duration};

/// Runtime configuration used by SDK clients when talking to Indigauge.
#[derive(Clone)]
pub struct IndigaugeConfig {
  api_base: String,
  game_name: String,
  public_key: String,
  game_version: String,
  batch_size: usize,
  flush_interval: Duration,
  max_queue: usize,
  request_timeout: Duration,
}

impl IndigaugeConfig {
  /// Creates a new config with sane defaults and optional env overrides.
  pub fn new(game_name: impl Into<String>, public_key: impl Into<String>, game_version: impl Into<String>) -> Self {
    Self {
      api_base: env::var("INDIGAUGE_API_BASE").unwrap_or_else(|_| "https://ingest.indigauge.com".into()),
      game_name: game_name.into(),
      public_key: public_key.into(),
      game_version: game_version.into(),
      batch_size: 64,
      flush_interval: Duration::from_secs(10),
      max_queue: 10_000,
      request_timeout: Duration::from_secs(10),
    }
  }

  /// Returns `true` if a non-empty public key is configured.
  pub fn has_public_key(&self) -> bool {
    !self.public_key.is_empty()
  }

  /// Returns the configured Indigauge public key.
  pub fn public_key(&self) -> &str {
    &self.public_key
  }

  /// Returns the configured game name.
  pub fn game_name(&self) -> &str {
    &self.game_name
  }

  /// Builds a versioned API URL for a relative path.
  pub fn api_url(&self, path: &str) -> String {
    format!("{}/v1/{}", &self.api_base, path)
  }

  /// Returns the ingest API base URL.
  pub fn api_base(&self) -> &str {
    &self.api_base
  }

  /// Returns the game version associated with this client.
  pub fn game_version(&self) -> &str {
    &self.game_version
  }

  /// Returns max number of events per batch request.
  pub fn batch_size(&self) -> usize {
    self.batch_size
  }

  /// Returns max in-memory queued events.
  pub fn max_queue(&self) -> usize {
    self.max_queue
  }

  /// Returns interval between periodic flush operations.
  pub fn flush_interval(&self) -> Duration {
    self.flush_interval
  }

  /// Returns request timeout used for outbound HTTP calls.
  pub fn request_timeout(&self) -> Duration {
    self.request_timeout
  }
}

#[derive(PartialEq, PartialOrd, Clone, Debug)]
pub enum IndigaugeLogLevel {
  Debug,
  Info,
  Warn,
  Error,
  Silent,
}

#[derive(PartialEq, Default, Clone, Debug)]
pub enum IndigaugeMode {
  /// Live mode sends data to the Indigauge API.
  #[default]
  Live,
  /// Dev mode only logs data to the console (if logging is enabled).
  Dev,
  /// Disabled mode does not send any data to the Indigauge API.
  Disabled,
}
