use bevy::prelude::*;
use indigauge_types::prelude::IndigaugeLogLevel;

use crate::{
  config::BevyIndigaugeLogLevel,
  event::resources::{BufferedEvents, EventQueueReceiver},
  session::resources::SessionApiKey,
  utils::BevyIndigauge,
};
use bevy::log::error;

/// Flushes buffered events immediately when batch size threshold is reached.
pub fn maybe_flush_events(mut ig: BevyIndigauge, session_key: Res<SessionApiKey>) {
  if ig.buffered_events.events.len() >= ig.config.batch_size() {
    ig.flush_events(&session_key);
  }
}

/// Periodic flush system that falls back to heartbeat when no events are pending.
pub fn flush_events(mut ig: BevyIndigauge, session_key: Res<SessionApiKey>) {
  if ig.flush_events(&session_key) == 0 {
    ig.send_heartbeat(&session_key);
  }
}

/// Moves events from the queue receiver into the in-memory batch buffer.
pub fn handle_queued_events(
  receiver: Res<EventQueueReceiver>,
  mut buffered_events: ResMut<BufferedEvents>,
  log_level: Res<BevyIndigaugeLogLevel>,
) {
  for event in receiver.try_iter() {
    match event.validate() {
      Ok(_) => {
        buffered_events.events.push(event);
      },
      Err(error) => {
        if **log_level <= IndigaugeLogLevel::Error {
          error!(message = "Invalid event", ?error);
        }
      },
    }
  }
}
