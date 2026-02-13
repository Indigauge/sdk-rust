use indigauge_types::prelude::{EventPayload, EventPayloadCtx, StartSessionResponse};
use serde_json::json;
use std::time::Instant;

/// Panic hook that ships a crash event and session end to the Indigauge backend.
/// Caller decides whether to run it (e.g., not in dev mode) and provides the session start instant.
pub fn panic_handler(
  host_origin: String,
  session_api_key: String,
  session_start: Instant,
) -> impl Fn(&std::panic::PanicHookInfo) + Send + Sync + 'static {
  move |info| {
    if session_api_key == StartSessionResponse::dev().session_token {
      return;
    }

    let elapsed_ms = Instant::now().duration_since(session_start).as_millis();

    let metadata = info
      .payload()
      .downcast_ref::<&str>()
      .map(|s| json!({"message": s.to_string()}));

    let context = info.location().map(|loc| EventPayloadCtx {
      file: loc.file().to_string(),
      line: loc.line(),
      module: None,
    });

    let payload = EventPayload {
      level: "fatal",
      event_type: "game.crash".to_string(),
      elapsed_ms,
      metadata,
      idempotency_key: None,
      context,
    };

    let single_event_endpoint = format!("{}/v1/events", host_origin);
    let client = reqwest::blocking::Client::new();
    let _ = client
      .post(&single_event_endpoint)
      .header("X-Indigauge-Key", &session_api_key)
      .json(&payload)
      .send();

    let end_session_endpoint = format!("{}/v1/sessions/end", host_origin);
    let _ = client
      .post(&end_session_endpoint)
      .header("X-Indigauge-Key", &session_api_key)
      .json(&json!({"reason": "panic"}))
      .send();
  }
}
