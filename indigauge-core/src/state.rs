use std::sync::OnceLock;
use std::time::Instant;
use std::{collections::VecDeque, sync::Mutex};

use crossbeam_channel::{Receiver, Sender, bounded};
use indigauge_types::prelude::{EventPayload, EventPayloadCtx};

use crate::event::{QueuedEvent, set_event_dispatcher};

pub(crate) static GLOBAL_TX: OnceLock<Sender<QueuedEvent>> = OnceLock::new();
pub(crate) static SESSION_START_INSTANT: OnceLock<Instant> = OnceLock::new();
pub(crate) static PENDING_EVENTS: OnceLock<Mutex<VecDeque<QueuedEvent>>> = OnceLock::new();

fn pending_events_lock() -> &'static Mutex<VecDeque<QueuedEvent>> {
  PENDING_EVENTS.get_or_init(|| Mutex::new(VecDeque::new()))
}

/// Tracks an enqueued event as pending delivery.
pub fn track_pending_event(event: QueuedEvent) {
  let mut pending = pending_events_lock()
    .lock()
    .unwrap_or_else(|poisoned| poisoned.into_inner());
  pending.push_back(event);
}

/// Clears up to `count` pending events from the front of the queue.
pub fn clear_pending_event_count(count: usize) {
  let mut pending = pending_events_lock()
    .lock()
    .unwrap_or_else(|poisoned| poisoned.into_inner());
  for _ in 0..count.min(pending.len()) {
    let _ = pending.pop_front();
  }
}

/// Drains and returns all currently tracked pending events.
pub fn drain_pending_events() -> Vec<QueuedEvent> {
  let mut pending = pending_events_lock()
    .lock()
    .unwrap_or_else(|poisoned| poisoned.into_inner());
  pending.drain(..).collect()
}

/// Initializes the Indigauge core state with a bounded channel.
/// Returns the receiver for processing queued events if initialization is successful.
pub fn init(max_queue: usize) -> Option<Receiver<QueuedEvent>> {
  if GLOBAL_TX.get().is_some() {
    return None;
  }

  let (tx, rx) = bounded(max_queue);
  if GLOBAL_TX.set(tx).is_ok() {
    set_event_dispatcher(enqueue);
    Some(rx)
  } else {
    None
  }
}

pub fn set_global_tx(tx: Sender<QueuedEvent>) -> Result<(), Sender<QueuedEvent>> {
  GLOBAL_TX.set(tx)
}

pub fn get_global_tx() -> Option<&'static Sender<QueuedEvent>> {
  GLOBAL_TX.get()
}

pub fn set_session_start_instant(instant: Instant) -> Result<(), Instant> {
  SESSION_START_INSTANT.set(instant)
}

pub fn get_session_start_instant() -> Option<&'static Instant> {
  SESSION_START_INSTANT.get()
}

#[inline]
/// Queues a validated event in the global sender if a session is active.
pub fn enqueue(
  level: &'static str,
  event_type: &str,
  metadata: Option<serde_json::Value>,
  file: &'static str,
  line: u32,
  module: &'static str,
) -> bool {
  let tx = match GLOBAL_TX.get() {
    Some(tx) => tx,
    None => return false,
  };

  if let Some(start_instant) = SESSION_START_INSTANT.get() {
    let elapsed_ms = Instant::now().duration_since(*start_instant).as_millis();
    let module = if module.is_empty() { None } else { Some(module) };

    let context = matches!(level, "warn" | "error").then(|| EventPayloadCtx {
      file: file.to_string(),
      line,
      module,
    });

    let payload = EventPayload::new(event_type, level, metadata, elapsed_ms).with_context(context);

    let queued_event = QueuedEvent::new(payload);
    let sent = tx.try_send(queued_event.clone()).is_ok();
    if sent {
      track_pending_event(queued_event);
    }
    sent
  } else {
    false
  }
}
