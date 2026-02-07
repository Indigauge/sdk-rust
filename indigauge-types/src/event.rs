use serde::Serialize;

#[derive(Serialize, Clone, Debug)]
pub struct BatchEventPayload {
  pub events: Vec<EventPayload>,
}

#[derive(Serialize, Clone, Debug, PartialEq)]
#[serde(rename_all = "camelCase")]
pub struct EventPayload {
  /// The type of the event. Event type must be in the format 'namespace.type'
  pub event_type: String,
  /// Metadata associated with the event.
  pub metadata: Option<serde_json::Value>,
  /// The level of the event.
  pub level: &'static str,
  /// Defaults to elapsed time since session start
  pub elapsed_ms: u128,
  /// Defaults to a random string
  pub idempotency_key: Option<String>,
  pub context: Option<EventPayloadCtx>,
}

#[derive(Serialize, Clone, Debug, PartialEq)]
pub struct EventPayloadCtx {
  pub file: String,
  pub line: u32,
  pub module: Option<&'static str>,
}
