use std::time::Instant;

use indigauge_core::prelude::{ig_debug_once, ig_error_once, ig_info_once, ig_trace_once, ig_warn_once};
use indigauge_core::state::{enqueue, init, set_session_start_instant, set_telemetry_consent};
use serde_json::json;

fn emit_once_events() {
  ig_trace_once!("once.trace");
  ig_debug_once!("once.debug");
  ig_info_once!("once.info", { "attempt": 1 });
  ig_warn_once!("once.warn");
  ig_error_once!("once.error");
}

#[test]
fn test_core_event_flow() {
  // 1. Init core system
  let rx = init(100).expect("Failed to init core");
  set_telemetry_consent(true);

  // 2. Set session start time (required for enqueue to work)
  set_session_start_instant(Instant::now()).expect("Failed to set start instant");

  // 3. Enqueue an event
  let queued = enqueue("info", "test.event", Some(json!({"foo": "bar"})), "test.rs", 10, "test_mod");
  assert!(queued, "Event should have been queued");

  // 4. consume event
  let event = rx.try_recv().expect("Should have received event");
  let payload = event.into_inner();

  assert_eq!(payload.event_type(), "test.event");
  assert_eq!(payload.level(), "info");

  emit_once_events();
  emit_once_events();

  let once_events = rx.try_iter().map(|event| event.into_inner()).collect::<Vec<_>>();
  assert_eq!(once_events.len(), 5);
  for event_type in ["once.trace", "once.debug", "once.info", "once.warn", "once.error"] {
    assert_eq!(
      once_events
        .iter()
        .filter(|event| event.event_type() == event_type)
        .count(),
      1
    );
  }
}
