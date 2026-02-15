# Indigauge Rust SDK

![Crates.io](https://img.shields.io/crates/v/bevy-mod-indigauge?logo=rust&label=crates.io)
![Docs.rs](https://img.shields.io/docsrs/bevy-mod-indigauge?logo=docs.rs&label=docs.rs)
![License](https://img.shields.io/github/license/Indigauge/sdk-rust)
![Bevy](https://img.shields.io/badge/Bevy-0.17-8A2BE2)

Instrument your game in minutes with the official Rust SDK for [Indigauge](https://www.indigauge.com).

Ship production-ready analytics, session health tracking, and player feedback pipelines without building your own telemetry stack.

This repository contains:

- A plug-and-play Bevy plugin for sessions, events, and feedback capture
- A shared core crate with fast event macros and optional tracing/panic hooks
- Consistent API payload models shared across all SDK crates

## Why teams use it

- **Faster launch:** add one plugin and start collecting meaningful game telemetry
- **Better player insight:** capture structured events and in-game feedback in one flow
- **Production-friendly:** batching, flush intervals, optional tracing, and panic reporting
- **Composable design:** use Bevy integration or low-level core/types crates directly

---

## Workspace Crates

| Crate | Purpose |
|---|---|
| [`bevy-mod-indigauge`](./bevy-mod-indigauge) | Bevy integration (plugin, session lifecycle, event queue, feedback UI) |
| [`indigauge-core`](./indigauge-core) | Core event macros and dispatch/tracing foundations |
| [`indigauge-types`](./indigauge-types) | Shared request/response payload models and enums |

---

## Quick Start (Bevy)

Add the Bevy plugin crate:

```toml
[dependencies]
bevy = "0.17"
bevy-mod-indigauge = "0.5"
```

Minimal integration:

```rust,no_run
use bevy::prelude::*;
use bevy_mod_indigauge::prelude::*;

fn main() {
	App::new()
		.add_plugins(DefaultPlugins)
		.add_plugins(
			IndigaugePlugin::<EmptySessionMeta>::new(
				"YOUR_PUBLIC_KEY",
				"My Game",
				env!("CARGO_PKG_VERSION"),
			)
			.mode(IndigaugeMode::Live)
			.log_level(IndigaugeLogLevel::Info),
		)
		.add_systems(Startup, setup)
		.run();
}

fn setup(mut commands: Commands) {
	commands.trigger(StartSessionEvent::new());
}
```

Send events with macros:

```rust,ignore
ig_info!("player.jump", { "height": 2.4 });
ig_warn!("network.retry", { "attempt": 2 });
ig_error!("matchmaking.timeout", { "region": "eu-west" });
```

---

## Crate-by-Crate Usage

### `bevy-mod-indigauge`

Use when you want plug-and-play Bevy support:

- Session start/heartbeat/end lifecycle
- Event buffering + periodic flush
- Optional in-game feedback UI
- Optional tracing integration (`tracing` feature)

See: [`bevy-mod-indigauge/README.md`](./bevy-mod-indigauge/README.md)

### `indigauge-core`

Use when you need core macros/dispatching without Bevy:

- Event macros (`ig_info!`, `ig_warn!`, `ig_error!`, ...)
- Compile-time event type validation (`namespace.event`)
- Optional tracing layer and panic handler

See: [`indigauge-core/README.md`](./indigauge-core/README.md)

### `indigauge-types`

Use for shared payload models:

- Event payloads and context models
- Session request/response models
- SDK-wide config/log-level/mode enums

See: [`indigauge-types/README.md`](./indigauge-types/README.md)

---

## Features

Notable optional features in the workspace:

- `bevy-mod-indigauge/tracing` — forwards tracing output to Indigauge
- `bevy-mod-indigauge/panic_handler` — sends crash event on panic (native)
- `bevy-mod-indigauge/feedback` — built-in feedback panel (default)
- `bevy-mod-indigauge/feedback_egui` — feedback panel rendered via `bevy_egui`

---

## Development

From the `sdk-rust` directory:

```bash
# Build all crates
cargo build --workspace

# Run tests
cargo test --workspace

# Lint (all targets + features)
cargo clippy --workspace --all-targets --all-features
```

Run Bevy plugin examples:

```bash
cargo run --release -p bevy-mod-indigauge --example minimal
cargo run --release -p bevy-mod-indigauge --example breakout
cargo run --release -p bevy-mod-indigauge --example feedback_egui --features feedback_egui
```

---

## Compatibility

- Bevy: `0.17`
- Rust edition: `2024` (crate-level)

---

## License

Dual-licensed under:

- MIT
- Apache-2.0

You may choose either license when using these crates.