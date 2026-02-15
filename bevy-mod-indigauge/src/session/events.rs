use bevy::prelude::*;

/// Emitted when session initialization completes or fails.
#[derive(Event, Debug)]
pub enum IndigaugeInitDoneEvent {
  Success,
  Skipped(String),
  Failure(String),
  UnexpectedFailure(String),
}

/// Event used to trigger session start.
#[derive(Event, Default, Clone)]
pub struct StartSessionEvent {
  pub platform: Option<String>,
}

impl StartSessionEvent {
  /// Creates a default start-session event.
  pub fn new() -> Self {
    Self::default()
  }

  /// Sets the platform string to send with session start payload.
  pub fn with_platform(mut self, platform: impl Into<String>) -> Self {
    self.platform = Some(platform.into());
    self
  }
}
