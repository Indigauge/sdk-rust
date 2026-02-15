use std::sync::OnceLock;

use indigauge_types::prelude::EventPayload;
use serde_json::Value;

/// Queued event with basic validation helpers.
#[derive(Clone, Debug)]
pub struct QueuedEvent {
  payload: EventPayload,
}

impl QueuedEvent {
  /// Creates a new queued event wrapper.
  pub fn new(payload: EventPayload) -> Self {
    Self { payload }
  }

  /// Unwraps and returns the inner event payload.
  pub fn into_inner(self) -> EventPayload {
    self.payload
  }

  /// Validates the event type format.
  pub fn validate(&self) -> Result<(), String> {
    let (ns, t) = self.payload.event_type().split_once('.').ok_or("Invalid event type")?;
    if ns.trim().is_empty() || t.trim().is_empty() {
      return Err("Invalid event type".to_string());
    }
    Ok(())
  }
}

type DispatchFn = fn(
  level: &'static str,
  event_type: &str,
  metadata: Option<Value>,
  file: &'static str,
  line: u32,
  module: &'static str,
) -> bool;

static DISPATCH: OnceLock<DispatchFn> = OnceLock::new();

/// Registers the event dispatcher used by the macros to send/queue events.
/// Can be set once; subsequent calls are ignored.
pub fn set_event_dispatcher(dispatch: DispatchFn) {
  let _ = DISPATCH.set(dispatch);
}

/// Dispatches an event through the registered dispatcher if available.
pub fn dispatch_event(
  level: &'static str,
  event_type: &str,
  metadata: Option<Value>,
  file: &'static str,
  line: u32,
  module: &'static str,
) -> bool {
  DISPATCH
    .get()
    .map(|dispatch| dispatch(level, event_type, metadata, file, line, module))
    .unwrap_or(false)
}

/// Runtime validation for event types.
pub const fn validate_event_type(s: &str) -> Result<(), &'static str> {
  let bytes = s.as_bytes();
  let len = bytes.len();

  if len < 3 {
    return Err("Invalid event type: too short (expected 'a.b')");
  }

  let mut dot_index: Option<usize> = None;
  let mut i = 0;

  while i < len {
    let b = bytes[i];
    match b {
      b'a'..=b'z' | b'A'..=b'Z' => {},
      b'.' => {
        if dot_index.is_some() {
          return Err("Invalid event type: multiple '.' found");
        }
        dot_index = Some(i);
      },
      _ => return Err("Invalid event type: only letters and a single '.' are allowed"),
    }
    i += 1;
  }

  let dot_pos = match dot_index {
    Some(p) => p,
    None => return Err("Invalid event type: must contain one '.'"),
  };

  if dot_pos == 0 || dot_pos == len - 1 {
    return Err("Invalid event type: '.' cannot be the first or last character");
  }

  Ok(())
}

/// Compile-time validation for event types used in macros.
pub const fn validate_event_type_compile_time(s: &str) -> &str {
  if let Err(err) = validate_event_type(s) {
    panic!("{}", err);
  }
  s
}

/// Public macros for emitting Indigauge events with optional JSON metadata.
pub mod macros {
  /// Low-level helper used by the `ig_*` macros to queue an event.
  ///
  /// Validates the `event_type` at compile time and forwards it to the dispatcher
  /// registered via `set_event_dispatcher`. If no dispatcher is registered, the
  /// macro is a no-op.
  ///
  /// # Examples
  /// ```rust,no_run
  /// use indigauge_core::{enqueue_ig_event, event::set_event_dispatcher};
  /// use serde_json::Value;
  ///
  /// fn dispatcher(_: &'static str, _: &str, _: Option<Value>, _: &'static str, _: u32, _: &'static str) -> bool {
  ///   true
  /// }
  ///
  /// fn main() {
  ///   set_event_dispatcher(dispatcher);
  ///   enqueue_ig_event!(info, "game.start", None);
  /// }
  /// ```
  #[macro_export]
  macro_rules! enqueue_ig_event {
    ($level: ident, $etype:expr, $metadata:expr) => {{
      const _VALID: &str = $crate::event::validate_event_type_compile_time($etype);
      let _ = $crate::event::dispatch_event(stringify!($level), $etype, $metadata, file!(), line!(), module_path!());
    }};
  }

  /// Emit an Indigauge event at the given level with optional structured metadata.
  ///
  /// Usage example: `ig_event!(info, "ui.click", { "button": btn_id, "x": x, "y": y });`
  ///
  /// The `event_type` must be a string literal in the form `"namespace.event"` and
  /// is compile-time validated to ensure it contains exactly one `.` and
  /// only letters on each side. Metadata is encoded as
  /// JSON using a shorthand object literal with string keys.
  #[macro_export]
  macro_rules! ig_event {
    ($level:ident, $etype:expr $(,)?) => {{
      $crate::enqueue_ig_event!($level, $etype, None);
    }};
    ($level:ident, $etype:expr $(, { $($key:tt : $value:expr),* $(,)? })? ) => {{
      let meta = serde_json::json!({ $($($key : $value),*)? });
      $crate::enqueue_ig_event!($level, $etype, Some(meta));
    }};
  }

  /// Logs or enqueues a **trace-level** event to Indigauge.
  ///
  /// # Format
  /// ```ignore
  /// ig_trace!(<event_type> [, { <metadata_key>: <value>, ... }]);
  /// ```
  ///
  /// * `<event_type>` — must be a string literal formatted as "namespace.event",
  ///   e.g. "ui.click", "gameplay.start". The value is compile-time validated
  ///   to ensure it contains exactly one `.` and only letters on each side.
  /// * Optional metadata can be passed as a JSON-like key/value list.
  ///
  /// # Examples
  /// ```ignore
  /// ig_trace!("ui.hover");
  /// ig_trace!("ui.hover", { "x": 128, "y": 256 });
  /// ```
  #[macro_export]
  macro_rules! ig_trace {
      ($($tt:tt)*) => { $crate::ig_event!(trace, $($tt)*); }
  }

  /// Logs or enqueues a **debug-level** event to Indigauge.
  ///
  /// # Format
  /// ```ignore
  /// ig_debug!(<event_type> [, { <metadata_key>: <value>, ... }]);
  /// ```
  ///
  /// * `<event_type>` — must be a string literal formatted as "namespace.event",
  ///   e.g. "ui.click", "gameplay.start". The value is compile-time validated
  ///   to ensure it contains exactly one `.` and only letters on each side.
  /// * Optional metadata can be passed as a JSON-like key/value list.
  ///
  /// # Examples
  /// ```ignore
  /// ig_debug!("system.update_start");
  /// ig_debug!("pathfinding.step", { "node": node_id, "distance": dist });
  /// ```
  #[macro_export]
  macro_rules! ig_debug {
      ($($tt:tt)*) => { $crate::ig_event!(debug, $($tt)*); }
  }

  /// Logs or enqueues an **info-level** event to Indigauge.
  ///
  /// Used for general application telemetry that represents normal operation.
  ///
  /// # Format
  /// ```ignore
  /// ig_info!(<event_type> [, { <metadata_key>: <value>, ... }]);
  /// ```
  ///
  /// * `<event_type>` — must be a string literal formatted as "namespace.event",
  ///   e.g. "ui.click", "gameplay.start". The value is compile-time validated
  ///   to ensure it contains exactly one `.` and only letters on each side.
  /// * Optional metadata can be passed as a JSON-like key/value list.
  ///
  /// # Examples
  /// ```ignore
  /// ig_info!("gameplay.start");
  /// ig_info!("gameplay.start", { "session": session_id });
  /// ig_info!("ui.click", { "button": "play" });
  /// ```
  #[macro_export]
  macro_rules! ig_info {
      ($($tt:tt)*) => { $crate::ig_event!(info, $($tt)*); }
  }

  /// Logs or enqueues a **warning-level** event to Indigauge.
  ///
  /// Used for non-fatal issues that should be visible in dashboards or analytics.
  ///
  /// # Format
  /// ```ignore
  /// ig_warn!(<event_type> [, { <metadata_key>: <value>, ... }]);
  /// ```
  ///
  /// * `<event_type>` — must be a string literal formatted as "namespace.event",
  ///   e.g. "ui.click", "gameplay.start". The value is compile-time validated
  ///   to ensure it contains exactly one `.` and only letters on each side.
  /// * Optional metadata can be passed as a JSON-like key/value list.
  ///
  /// # Examples
  /// ```ignore
  /// ig_warn!("network.latency", { "ping_ms": latency });
  /// ig_warn!("save.failed", { "reason": "disk_full" });
  /// ```
  #[macro_export]
  macro_rules! ig_warn {
      ($($tt:tt)*) => { $crate::ig_event!(warn, $($tt)*); }
  }

  /// Logs or enqueues an **error-level** event to Indigauge.
  ///
  /// Used to capture errors, failures, or critical analytics signals.
  ///
  /// # Format
  /// ```ignore
  /// ig_error!(<event_type> [, { <metadata_key>: <value>, ... }]);
  /// ```
  ///
  /// * `<event_type>` — must be a string literal formatted as "namespace.event",
  ///   e.g. "ui.click", "gameplay.start". The value is compile-time validated
  ///   to ensure it contains exactly one `.` and only letters on each side.
  /// * Optional metadata can be passed as a JSON-like key/value list.
  ///
  /// # Examples
  /// ```ignore
  /// ig_error!("game.crash", { "reason": error_message });
  /// ig_error!("network.disconnect");
  /// ```
  ///
  /// The metadata is optional, but recommended for debugging or later filtering.
  #[macro_export]
  macro_rules! ig_error {
      ($($tt:tt)*) => { $crate::ig_event!(error, $($tt)*); }
  }
}

#[cfg(test)]
mod tests {
  use super::*;

  #[test]
  fn validates_event_type() {
    assert!(validate_event_type("game.start").is_ok());
    assert!(validate_event_type("game").is_err());
    assert!(validate_event_type(".start").is_err());
    assert!(validate_event_type("game.").is_err());
    assert!(validate_event_type("game..start").is_err());
  }
}
