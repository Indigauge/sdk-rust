use bevy::prelude::*;
use indigauge_core::types::IndigaugeLogLevel;

use crate::{
  config::BevyIndigaugeLogLevel,
  event::resources::{BufferedEvents, EventPostingDisabled, EventQueueReceiver},
  session::resources::SessionApiKey,
  utils::BevyIndigauge,
};
use bevy::log::error;

/// Flushes buffered events immediately when batch size threshold is reached.
pub fn maybe_flush_events(
  mut ig: BevyIndigauge,
  session_key: Option<Res<SessionApiKey>>,
  event_posting_disabled: Res<EventPostingDisabled>,
) {
  if event_posting_disabled.0 {
    ig.buffered_events.events.clear();
    return;
  }

  if let Some(key) = session_key
    && ig.buffered_events.events.len() >= ig.config.batch_size()
  {
    ig.flush_events(&key);
  }
}

/// Periodic flush system that falls back to heartbeat when no events are pending.
pub fn flush_events(
  mut ig: BevyIndigauge,
  session_key: Option<Res<SessionApiKey>>,
  event_posting_disabled: Res<EventPostingDisabled>,
) {
  if event_posting_disabled.0 {
    ig.buffered_events.events.clear();
    if let Some(key) = session_key {
      ig.send_heartbeat(&key);
    }
    return;
  }

  if let Some(key) = session_key
    && ig.flush_events(&key) == 0
  {
    ig.send_heartbeat(&key);
  }
}

/// Moves events from the queue receiver into the in-memory batch buffer.
pub fn handle_queued_events(
  receiver: Res<EventQueueReceiver>,
  mut buffered_events: ResMut<BufferedEvents>,
  log_level: Res<BevyIndigaugeLogLevel>,
  event_posting_disabled: Res<EventPostingDisabled>,
) {
  if event_posting_disabled.0 {
    buffered_events.events.clear();
    for _ in receiver.try_iter() {} // drain the channel
    return;
  }

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
