#[cfg(not(target_family = "wasm"))]
use crate::runtime::IndigaugeBlockingRuntimeClient;
#[cfg(not(target_family = "wasm"))]
use crate::state::drain_pending_events;
#[cfg(not(target_family = "wasm"))]
use crate::types::BatchEventPayload;
use indigauge_types::prelude::IndigaugeConfig;
#[cfg(not(target_family = "wasm"))]
use indigauge_types::prelude::{EventPayload, EventPayloadCtx, StartSessionResponse};
#[cfg(not(target_family = "wasm"))]
use serde_json::json;
use std::time::Instant;

/// Panic hook that ships a crash event and session end to the Indigauge backend.
/// Caller decides whether to run it (e.g., not in dev mode) and provides the session start instant.
#[cfg(not(target_family = "wasm"))]
pub fn panic_handler_with_config(
  config: IndigaugeConfig,
  session_api_key: String,
  session_start: Instant,
) -> impl Fn(&std::panic::PanicHookInfo) + Send + Sync + 'static {
  let sdk_client = IndigaugeBlockingRuntimeClient::new(config);

  move |info| {
    if session_api_key == StartSessionResponse::dev().session_token {
      return;
    }

    let pending_events = drain_pending_events()
      .into_iter()
      .map(|event| event.into_inner())
      .collect::<Vec<_>>();

    if !pending_events.is_empty() {
      let payload = BatchEventPayload { events: pending_events };

      if let Ok(request) = sdk_client.event_batch(&session_api_key, &payload) {
        let _ = sdk_client.send(request);
      }
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

    let _payload = EventPayload::new("game.crash", "fatal", metadata, elapsed_ms).with_context(context);

    if let Ok(request) = sdk_client.end_session(&session_api_key, "crashed") {
      let _ = sdk_client.send(request);
    }
  }
}

#[cfg(target_family = "wasm")]
pub fn panic_handler_with_config(
  config: IndigaugeConfig,
  session_api_key: String,
  session_start: Instant,
) -> impl Fn(&std::panic::PanicHookInfo) + Send + Sync + 'static {
  let _ = (config, session_api_key, session_start);
  move |_info| {}
}

/// Legacy panic hook constructor using explicit API origin.
/// Prefer [`panic_handler_with_config`] when possible.
#[cfg(not(target_family = "wasm"))]
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

    let payload = EventPayload::new("game.crash", "fatal", metadata, elapsed_ms).with_context(context);

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
      .json(&json!({"reason": "crashed"}))
      .send();
  }
}

#[cfg(target_family = "wasm")]
pub fn panic_handler(
  host_origin: String,
  session_api_key: String,
  session_start: Instant,
) -> impl Fn(&std::panic::PanicHookInfo) + Send + Sync + 'static {
  let _ = (host_origin, session_api_key, session_start);
  move |_info| {}
}
