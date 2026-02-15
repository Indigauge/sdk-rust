# Indigauge Types

Shared data structures for the Indigauge Rust SDK. These types are used by the other crates to describe events, sessions, feedback, and configuration in a single, consistent place.

## What you get

- Event primitives (`EventPayload`, `EventPayloadCtx`) used when sending structured events.
- Log and mode enums (`IndigaugeLogLevel`, `IndigaugeMode`) shared across the SDK.
- Configuration and session models for client-side integrations.
- Optional feedback-related models when the `feedback` feature is enabled.

## Features

- `feedback` (default): includes feedback UI payload types. Disable for minimal builds that do not surface feedback functionality.

## Examples

Creating an event payload manually (usually handled by higher-level crates):

```rust
use indigauge_types::prelude::{EventPayload, EventPayloadCtx};

let payload = EventPayload::new("game.start", "info", None, 1)
  .with_context(Some(EventPayloadCtx {
    file: "main.rs".to_string(),
    line: 42,
    module: Some("game"),
  }));

println!("payload: {payload:?}");
```

## License

Dual-licensed under MIT or Apache-2.0. You may choose either license when using this crate.
