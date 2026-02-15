use serde::Serialize;
use uuid::Uuid;

/// Batch payload for sending multiple events in a single request.
#[derive(Serialize, Clone, Debug)]
pub struct BatchEventPayload {
  pub events: Vec<EventPayload>,
}

/// Structured event payload sent to Indigauge ingest endpoints.
#[derive(Serialize, Clone, Debug, PartialEq)]
#[serde(rename_all = "camelCase")]
pub struct EventPayload {
  /// The type of the event. Event type must be in the format 'namespace.type'
  event_type: String,
  /// Metadata associated with the event.
  metadata: Option<serde_json::Value>,
  /// The level of the event.
  level: &'static str,
  /// Defaults to elapsed time since session start
  elapsed_ms: u128,
  idempotency_key: String,
  context: Option<EventPayloadCtx>,
}

impl EventPayload {
  /// Creates a new event payload and generates an idempotency key.
  pub fn new(
    event_type: impl Into<String>,
    level: &'static str,
    metadata: Option<serde_json::Value>,
    elapsed_ms: u128,
  ) -> Self {
    Self {
      event_type: event_type.into(),
      level,
      metadata,
      elapsed_ms,
      idempotency_key: Uuid::new_v4().to_string(),
      context: None,
    }
  }

  /// Attaches optional source context (file/line/module) to the payload.
  pub fn with_context(mut self, ctx: Option<EventPayloadCtx>) -> Self {
    self.context = ctx;
    self
  }

  /// Returns metadata if present.
  pub fn metadata(&self) -> Option<&serde_json::Value> {
    self.metadata.as_ref()
  }

  /// Returns the event severity level.
  pub fn level(&self) -> &str {
    self.level
  }

  /// Returns the event type in `namespace.event` format.
  pub fn event_type(&self) -> &str {
    &self.event_type
  }

  /// Returns context information if present.
  pub fn context(&self) -> Option<&EventPayloadCtx> {
    self.context.as_ref()
  }
}

/// Source code context for an event.
#[derive(Serialize, Clone, Debug, PartialEq)]
pub struct EventPayloadCtx {
  pub file: String,
  pub line: u32,
  pub module: Option<&'static str>,
}
