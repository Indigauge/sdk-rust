# Indigauge Core

Foundation crate for the Indigauge Rust SDK. It provides the shared building blocks that other crates build on:

- Event macros (`ig_info!`, `ig_warn!`, `ig_error!`, etc.) with compile-time validation of `namespace.event` names.
- Pluggable dispatcher so host environments (Bevy, servers, tools) decide how events are queued or sent.
- Optional tracing layer that forwards tracing spans/events to Indigauge.
- Optional panic handler that captures crashes as events (native targets).
- Hardware helpers (CPU/RAM bucketing, CPU name coarsening) for lightweight device context.
- Framework-agnostic runtime clients (`IndigaugeRuntimeClient` async + `IndigaugeBlockingRuntimeClient` native blocking).

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

## Framework-agnostic runtime usage

Use `indigauge-core` directly when integrating with engines beyond Bevy (e.g. ggez, macroquad, Fyrox).

```rust,no_run
use indigauge_core::runtime::IndigaugeBlockingRuntimeClient;
use indigauge_core::types::{EventPayload, IndigaugeConfig};

let config = IndigaugeConfig::new("My Game", "PUBLIC_KEY", "1.0.0");
let sdk = IndigaugeBlockingRuntimeClient::new(config);

let event = EventPayload::new("game.start", "info", None, 0);
let request = sdk.event("SESSION_TOKEN", &event)?;
let _response = sdk.send(request)?;
# Ok::<(), Box<dyn std::error::Error>>(())
```

## License

Dual-licensed under MIT or Apache-2.0.
