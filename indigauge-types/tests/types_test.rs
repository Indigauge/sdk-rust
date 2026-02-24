use indigauge_types::prelude::*;
use serde_json::json;

#[test]
fn test_event_payload_serialization() {
  let payload = EventPayload::new("test.foo", "info", Some(json!({"x": 1})), 123);
  let json = serde_json::to_string(&payload).expect("Serialize");

  assert!(json.contains(r#""eventType":"test.foo""#));
  assert!(json.contains(r#""level":"info""#));
  assert!(json.contains(r#""x":1"#));
}

#[test]
fn test_session_start_serialization() {
  let session = StartSessionPayload {
    client_version: "1.0.0",
    sdk_version: "2.0.0",
    player_id: Some(&"player1".to_string()),
    platform: None,
    os: None,
    cpu_family: None,
    cores: None,
    memory: None,
    gpu: None,
  };

  let json = serde_json::to_string(&session).expect("Serialize");
  assert!(json.contains(r#""clientVersion":"1.0.0""#));
  assert!(json.contains(r#""sdkVersion":"2.0.0""#));
}
