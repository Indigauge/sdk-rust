use std::marker::PhantomData;

use bevy::prelude::*;
use indigauge_core::state::init;
use indigauge_core::types::{IndigaugeLogLevel, IndigaugeMode};
use serde::Serialize;

use crate::{
  config::*,
  event::{
    EventsPlugin,
    resources::{BufferedEvents, EventQueueReceiver},
  },
  http_runtime::ReqwestPlugin,
  session::{SessionPlugin, resources::EmptySessionMeta},
};
use bevy::log::{info, warn};

/// Main Bevy plugin entrypoint for Indigauge telemetry and feedback features.
pub struct IndigaugePlugin<Meta = EmptySessionMeta> {
  public_key: String,
  /// Defaults to cargo package name
  game_name: String,
  game_version: String,
  log_level: BevyIndigaugeLogLevel,
  mode: BevyIndigaugeMode,
  meta: PhantomData<Meta>,
}

impl<M> IndigaugePlugin<M> {
  /// Sets the SDK log level resource.
  pub fn log_level(mut self, log_level: IndigaugeLogLevel) -> Self {
    self.log_level = BevyIndigaugeLogLevel(log_level);
    self
  }

  /// Sets the SDK runtime mode.
  pub fn mode(mut self, mode: IndigaugeMode) -> Self {
    self.mode = BevyIndigaugeMode(mode);
    self
  }
}

impl<M> IndigaugePlugin<M>
where
  M: Resource + Serialize,
{
  /// Creates a plugin configured with the provided game identity and key.
  pub fn new(public_key: impl Into<String>, game_name: impl Into<String>, game_version: impl Into<String>) -> Self {
    Self {
      public_key: public_key.into(),
      game_name: game_name.into(),
      game_version: game_version.into(),
      ..Default::default()
    }
  }
}

impl<M> Default for IndigaugePlugin<M>
where
  M: Resource + Serialize,
{
  fn default() -> Self {
    Self {
      game_name: env!("CARGO_PKG_NAME").to_string(),
      public_key: std::env::var("INDIGAUGE_PUBLIC_KEY").unwrap_or_default(),
      game_version: env!("CARGO_PKG_VERSION").to_string(),
      log_level: BevyIndigaugeLogLevel(IndigaugeLogLevel::Info),
      mode: BevyIndigaugeMode::default(),
      meta: PhantomData,
    }
  }
}

impl<M> Plugin for IndigaugePlugin<M>
where
  M: Resource + Serialize,
{
  fn build(&self, app: &mut App) {
    let config = BevyIndigaugeConfig::new(&self.game_name, &self.public_key, &self.game_version);

    if matches!(*self.mode, IndigaugeMode::Live | IndigaugeMode::Dev) {
      if !config.has_public_key() && *self.mode == IndigaugeMode::Live {
        if *self.log_level <= IndigaugeLogLevel::Warn {
          warn!(
            "Indigauge public key is not set for live-mode. Please set the INDIGAUGE_PUBLIC_KEY environment variable to start sessions and send events."
          );
        }
      } else if let Some(rx) = init(config.max_queue()) {
        if !config.has_public_key() && *self.log_level <= IndigaugeLogLevel::Info {
          info!(
            "Indigauge public key is not set for dev-mode. Logs will still be shown in the console, but not sent to the server."
          );
        }
        app.insert_resource(EventQueueReceiver::new(rx));
      }
    }

    #[cfg(feature = "feedback")]
    app.add_plugins(crate::feedback::FeedbackUiPlugin);

    app
      .add_plugins(ReqwestPlugin)
      .add_plugins((EventsPlugin::new(config.flush_interval()), SessionPlugin::<M>::new(config.flush_interval())))
      .insert_resource(self.log_level.clone())
      .insert_resource(BufferedEvents::default())
      .insert_resource(self.mode.clone())
      .insert_resource(config);
  }
}
