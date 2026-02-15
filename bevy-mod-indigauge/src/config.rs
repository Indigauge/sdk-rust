use indigauge_types::prelude::{IndigaugeConfig, IndigaugeLogLevel, IndigaugeMode};

use bevy::prelude::*;

/// Bevy resource wrapper around the shared Indigauge SDK config.
#[derive(Resource, Clone, Deref)]
pub struct BevyIndigaugeConfig(pub IndigaugeConfig);

impl BevyIndigaugeConfig {
  /// Creates a new Bevy config resource from game identity values.
  pub fn new(game_name: impl Into<String>, public_key: impl Into<String>, game_version: impl Into<String>) -> Self {
    Self(IndigaugeConfig::new(game_name, public_key, game_version))
  }
}

/// Bevy resource wrapper for log verbosity.
#[derive(Resource, Clone, Deref, Debug)]
pub struct BevyIndigaugeLogLevel(pub IndigaugeLogLevel);

/// Bevy resource wrapper for SDK runtime mode.
#[derive(Resource, Default, Clone, Deref, Debug)]
pub struct BevyIndigaugeMode(pub IndigaugeMode);
