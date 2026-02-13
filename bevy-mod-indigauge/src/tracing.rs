use crate::config::BevyIndigaugeLogLevel;
use crate::event::utils::enqueue;
use bevy::utils::tracing::Level;
use indigauge_core::tracing::IndigaugeSink;
use indigauge_types::prelude::IndigaugeLogLevel;
use std::sync::Arc;

pub use indigauge_core::tracing::IndigaugeLayer;

impl BevyIndigaugeLogLevel {
  pub fn as_str(&self) -> &'static str {
    match **self {
      IndigaugeLogLevel::Debug => "debug",
      IndigaugeLogLevel::Info => "info",
      IndigaugeLogLevel::Warn => "warn",
      IndigaugeLogLevel::Error => "error",
      IndigaugeLogLevel::Silent => "silent",
    }
  }
}

impl From<&Level> for BevyIndigaugeLogLevel {
  fn from(level: &Level) -> Self {
    match *level {
      Level::ERROR => BevyIndigaugeLogLevel(IndigaugeLogLevel::Error),
      Level::WARN => BevyIndigaugeLogLevel(IndigaugeLogLevel::Warn),
      Level::INFO => BevyIndigaugeLogLevel(IndigaugeLogLevel::Info),
      Level::DEBUG => BevyIndigaugeLogLevel(IndigaugeLogLevel::Debug),
      Level::TRACE => BevyIndigaugeLogLevel(IndigaugeLogLevel::Silent),
    }
  }
}

struct EnqueueSink;

impl IndigaugeSink for EnqueueSink {
  fn log(
    &self,
    level: &'static str,
    event_type: &str,
    metadata: Option<serde_json::Value>,
    file: &'static str,
    line: u32,
    module: &'static str,
  ) {
    enqueue(level, event_type, metadata, file, line, module);
  }
}

pub fn default_bevy_indigauge_layer() -> IndigaugeLayer {
  // Start from core layer and keep existing defaults (filters and levels) while logging via enqueue.
  IndigaugeLayer::new_with_sink(Arc::new(EnqueueSink))
}
