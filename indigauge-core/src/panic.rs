#[cfg(not(target_family = "wasm"))]
use crate::runtime::IndigaugeBlockingRuntimeClient;
#[cfg(target_family = "wasm")]
use crate::runtime::IndigaugeRuntimeClient;

use crate::state::drain_pending_events;
use crate::types::BatchEventPayload;
use indigauge_types::prelude::IndigaugeConfig;
use indigauge_types::prelude::{DEV_SESSION_TOKEN, EventPayload, EventPayloadCtx};
use serde_json::json;
use std::panic::PanicHookInfo;
use std::time::Instant;
#[cfg(target_family = "wasm")]
use wasm_bindgen_futures::spawn_local;

fn send_pending_events_payload(info: &PanicHookInfo<'_>, session_start: Instant) -> BatchEventPayload {
  let mut pending_events = drain_pending_events()
    .into_iter()
    .map(|event| event.into_inner())
    .collect::<Vec<_>>();

  let last_event_payload = end_session_payload(info, session_start);
  pending_events.push(last_event_payload);

  BatchEventPayload { events: pending_events }
}

fn end_session_payload(info: &PanicHookInfo<'_>, session_start: Instant) -> EventPayload {
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

  EventPayload::new("game.crash", "fatal", metadata, elapsed_ms).with_context(context)
}

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
    if session_api_key == DEV_SESSION_TOKEN {
      return;
    }

    let payload = send_pending_events_payload(info, session_start);
    if let Ok(request) = sdk_client.event_batch(&session_api_key, &payload) {
      let _ = sdk_client.send(request);
    }

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
  let sdk_client = IndigaugeRuntimeClient::new(config);

  move |info| {
    if session_api_key == DEV_SESSION_TOKEN {
      return;
    }

    let payload = send_pending_events_payload(info, session_start);
    let batch_request = sdk_client.event_batch(&session_api_key, &payload).ok();
    let end_request = sdk_client.end_session(&session_api_key, "crashed").ok();

    if batch_request.is_none() && end_request.is_none() {
      return;
    }

    let client = sdk_client.client().clone();
    spawn_local(async move {
      if let Some(request) = batch_request {
        let _ = client.execute(request).await;
      }

      if let Some(request) = end_request {
        let _ = client.execute(request).await;
      }
    });
  }
}
