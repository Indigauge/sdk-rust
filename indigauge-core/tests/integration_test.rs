use std::time::Instant;

use indigauge_core::state::{enqueue, init, set_session_start_instant};
use serde_json::json;

#[test]
fn test_core_event_flow() {
  // 1. Init core system
  let rx = init(100).expect("Failed to init core");

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
}
