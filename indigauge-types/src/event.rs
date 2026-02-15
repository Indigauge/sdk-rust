use serde::Serialize;
use uuid::Uuid;

#[derive(Serialize, Clone, Debug)]
pub struct BatchEventPayload {
  pub events: Vec<EventPayload>,
}

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

  pub fn with_context(mut self, ctx: Option<EventPayloadCtx>) -> Self {
    self.context = ctx;
    self
  }

  pub fn metadata(&self) -> Option<&serde_json::Value> {
    self.metadata.as_ref()
  }

  pub fn level(&self) -> &str {
    self.level
  }

  pub fn event_type(&self) -> &str {
    &self.event_type
  }

  pub fn context(&self) -> Option<&EventPayloadCtx> {
    self.context.as_ref()
  }
}

#[derive(Serialize, Clone, Debug, PartialEq)]
pub struct EventPayloadCtx {
  pub file: String,
  pub line: u32,
  pub module: Option<&'static str>,
}
