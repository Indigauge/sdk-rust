use std::time::Instant;

use crate::{plugin::GLOBAL_TX, session::SESSION_START_INSTANT};
use indigauge_core::event::QueuedEvent;
use indigauge_types::prelude::{EventPayload, EventPayloadCtx};

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
    Some(tx) => tx.clone(),
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

    tx.try_send(QueuedEvent::new(payload)).is_ok()
  } else {
    false
  }
}
