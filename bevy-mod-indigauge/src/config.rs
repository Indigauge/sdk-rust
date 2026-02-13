use indigauge_types::prelude::{IndigaugeConfig, IndigaugeLogLevel, IndigaugeMode};

use bevy::prelude::*;

#[derive(Resource, Clone, Deref)]
pub struct BevyIndigaugeConfig(pub IndigaugeConfig);

impl BevyIndigaugeConfig {
  pub fn new(game_name: impl Into<String>, public_key: impl Into<String>, game_version: impl Into<String>) -> Self {
    Self(IndigaugeConfig::new(game_name, public_key, game_version))
  }
}

#[derive(Resource, Clone, Deref, Debug)]
pub struct BevyIndigaugeLogLevel(pub IndigaugeLogLevel);

#[derive(Resource, Default, Clone, Deref, Debug)]
pub struct BevyIndigaugeMode(pub IndigaugeMode);
