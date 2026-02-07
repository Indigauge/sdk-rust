use indigauge_types::prelude::{IndigaugeConfig, IndigaugeLogLevel, IndigaugeMode};

use bevy::prelude::*;

#[derive(Resource, Clone, Deref)]
pub struct BevyIndigaugeConfig(pub IndigaugeConfig);

impl BevyIndigaugeConfig {
  pub fn new(game_name: impl Into<String>, public_key: impl Into<String>, game_version: impl Into<String>) -> Self {
    Self(IndigaugeConfig::new(game_name, public_key, game_version))
  }
}

#[derive(Resource, Clone, Deref)]
pub struct BevyIndigaugeLogLevel(pub IndigaugeLogLevel);

#[cfg(feature = "tracing")]
use bevy::utils::tracing::Level;

#[cfg(feature = "tracing")]
impl From<&Level> for IndigaugeLogLevel {
  fn from(level: &Level) -> Self {
    match *level {
      Level::ERROR => BevyIndigaugeLogLevel::Error,
      Level::WARN => BevyIndigaugeLogLevel::Warn,
      Level::INFO => BevyIndigaugeLogLevel::Info,
      Level::DEBUG => BevyIndigaugeLogLevel::Debug,
      Level::TRACE => BevyIndigaugeLogLevel::Trace,
    }
  }
}

#[derive(Resource, Default, Clone, Deref)]
pub struct BevyIndigaugeMode(pub IndigaugeMode);
