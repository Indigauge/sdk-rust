# Indigauge Core

Foundation crate for the Indigauge Rust SDK. It provides the shared building blocks that other crates build on:

- Event macros (`ig_info!`, `ig_warn!`, `ig_error!`, etc.) with compile-time validation of `namespace.event` names.
- Pluggable dispatcher so host environments (Bevy, servers, tools) decide how events are queued or sent.
- Optional tracing layer that forwards tracing spans/events to Indigauge.
- Optional panic handler that captures crashes as events (native targets).
- Hardware helpers (CPU/RAM bucketing, CPU name coarsening) for lightweight device context.

## Quick start

```rust,ignore
use indigauge_core::{ig_info, set_event_dispatcher};
use serde_json::Value;

fn dispatcher(_level: &'static str, etype: &str, meta: Option<Value>, _file: &'static str, _line: u32, _module: &'static str) -> bool {
  println!("Dispatch {etype} with meta: {meta:?}");
  true
}

fn main() {
  set_event_dispatcher(dispatcher);
  ig_info!("game.start", { "build": "dev" });
}
```

## Features

- `panic_handler` — capture panics as events (native targets only).
- `tracing` — expose `IndigaugeLayer` and `IndigaugeSink` to ship tracing events.

## License

Dual-licensed under MIT or Apache-2.0.
