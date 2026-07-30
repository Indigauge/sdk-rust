# Indigauge Game SDK (Bevy)

The **Indigauge Game SDK** is a lightweight Rust library for sending structured analytics and player feedback from your **Bevy** games to the [Indigauge](https://www.indigauge.com) API.

It’s designed to be easy to integrate and is powerful enough for production use in indie games.

---

## Features

- **Bevy 0.19 compatible** — easy drop-in plugin
- Lightweight event macros: `ig_info!`, `ig_warn!`, `ig_error!`, …
- Built-in **Feedback UI panel** for in-game bug reports & suggestions
- Works on both **native** and **WASM** builds*
- **Tracing support** — log events to the Indigauge API through tracing

> [!WARNING]
> On wasm builds, the panic handler is disabled. No crash reports will be sent as events to the Indigauge API.

---

## Installation

```toml
[dependencies]
bevy = "0.19"
bevy-mod-indigauge = { version = "0.7" }
```

### Feature flags

- `consent` (default): telemetry consent state + consent modal UI.
- `feedback` (default): feedback panel rendered with Bevy UI + picking.
- `feedback_egui` (optional): render feedback panel with `bevy_egui`.

```toml
# Keep default consent + feedback UI features
bevy-mod-indigauge = { version = "0.7" }

# Enable optional bevy_egui feedback panel backend (native only)
bevy-mod-indigauge = { version = "0.7", features = ["feedback_egui"] }
```

## Examples

- [`minimal`](examples/minimal.rs) - An example showing start session, sending info events and triggering feedback form.
- [`breakout`](examples/breakout.rs) – An example showing a more realistic setup with a real game and game states.
- [`flappy_bird`](examples/flappy_bird.rs) - A compact Flappy Bird clone showing gameplay events, score tracking, and feedback prompts.
- [`feedback_egui`](examples/feedback_egui.rs) - A focused example showing the `bevy_egui` feedback panel triggers and props.

### Running Examples

```bash
cargo run --release --example minimal
```

```bash
cargo run --release --example breakout

# Or with tracing feature:
cargo run --release --example breakout --features tracing

# Run the flappy bird demo
cargo run --release --example flappy_bird

# Run the bevy_egui feedback demo (optional feature)
cargo run --release --example feedback_egui --features feedback_egui
```

## Quick Start

* Setup game project [Indigauge](https://www.indigauge.com)
* Create a public key for the game.
* Add the plugin to your game.

```rust,no_run
use std::time::Duration;
use bevy::{prelude::*, time::common_conditions::on_timer};
use bevy_mod_indigauge::prelude::*;

fn main() {
  App::new()
    .add_plugins(DefaultPlugins)
    .add_plugins(
      IndigaugePlugin::<EmptySessionMeta>::new(
        "YOUR_PUBLIC_KEY",
        "My game name",
        env!("CARGO_PKG_VERSION")
      )
      // Optional: Set mode (Defaults to live). Dev mode is useful for testing and debugging and does not send events to the server.
      .mode(IndigaugeMode::Dev)
      // Optional: Set preferred log-level (Defaults to Info)
      .log_level(IndigaugeLogLevel::Info)
    )
    .add_systems(Startup, setup)
    .add_systems(Update, (trigger_feedback_with_question, track_counter.run_if(on_timer(Duration::from_secs(2)))))
    .run();
}

fn setup(mut commands: Commands) {
  commands.spawn((Camera2d, IsDefaultUiCamera));
  commands.trigger(StartSessionEvent::new());
}

fn trigger_feedback_with_question(
  mut commands: Commands,
  keys: Res<ButtonInput<KeyCode>>,
) {
  if keys.just_pressed(KeyCode::KeyF) {
    // This is how you manually trigger the feedback panel
    commands.insert_resource(
      FeedbackPanelProps::with_question("What did you think about level 3?", FeedbackCategory::Gameplay),
    );
  }
}

fn track_counter(mut counter: Local<u32>) {
  *counter += 1;
  ig_info!("counter.increase", { "value": *counter });
}
```

## Sending events

Send structured events with macros. The events will only be sent if a session was successfully started.

```rust,ignore
ig_info!("player.jump", { "height": 2.4 });
ig_error!("physics.failed", { "component": "rigid_body" });
```

## Consent modal

You can optionally show a built-in consent modal before starting telemetry. The modal is fully customizable (style + copy), and the selected choice is stored in [`IndigaugeConsentState`].

The SDK enforces consent at runtime:

- No session is started unless consent is explicitly `Accepted`.
- If consent is `Declined` (or feature `consent` is disabled), telemetry transmission is blocked.
- If consent is revoked after acceptance, active session credentials and queued telemetry are discarded.

- `IndigaugeConsentChoice::Accepted` means the player allowed telemetry.
- `IndigaugeConsentChoice::Declined` means the player declined telemetry.
- `IndigaugeConsentChoice::Unknown` means no choice has been made yet.

For GDPR hardening, automatic session-start payload fields that can increase fingerprinting risk are no longer sent (`player_id`, `platform`, `os`, `cpu_family`, `cores`, `memory`, `gpu`).
Additionally, feedback screenshot upload is disabled.

On native targets, consent can be persisted automatically in the user preference folder (`dirs::preference_dir()/GAME_NAME/telemetry_consent.txt`).

### Basic flow

```rust,no_run
use bevy::prelude::*;
use bevy_mod_indigauge::prelude::*;

fn main() {
  App::new()
    .add_plugins(DefaultPlugins)
    .add_plugins(IndigaugePlugin::<EmptySessionMeta>::new("YOUR_PUBLIC_KEY", "My game", env!("CARGO_PKG_VERSION")))
    .add_systems(Startup, show_consent_if_needed)
    .add_observer(on_consent_selected)
    .run();
}

fn show_consent_if_needed(mut commands: Commands, consent: Res<IndigaugeConsentState>) {
  if consent.choice == IndigaugeConsentChoice::Unknown {
    commands.insert_resource(ConsentModalProps::default());
  }
}

fn on_consent_selected(trigger: On<IndigaugeConsentSelectedEvent>, mut commands: Commands) {
  if trigger.event().choice == IndigaugeConsentChoice::Accepted {
    commands.trigger(StartSessionEvent::default());
  }
}
```

### Customize modal copy and placement

```rust,no_run
use bevy::prelude::*;
use bevy_mod_indigauge::prelude::*;

fn show_custom_consent_modal(mut commands: Commands) {
  commands.insert_resource(
    ConsentModalProps::new()
      .title("Help us improve gameplay")
      .message("Allow anonymous telemetry so we can tune balance, fix crashes, and improve performance.")
      .accept_button_text("Allow telemetry")
      .decline_button_text("No thanks")
      .persist_choice(true)
      .spawn_position(ConsentModalSpawnPosition::Center)
      .margin(UiRect::all(Val::Px(20.0))),
  );
}
```

### Customize modal visual style

```rust,no_run
use bevy::prelude::*;
use bevy_mod_indigauge::prelude::*;

fn setup_consent_theme(mut commands: Commands) {
  commands.insert_resource(ConsentModalStyles {
    overlay: Color::srgba_u8(10, 15, 26, 214),
    background: Color::srgb_u8(15, 23, 42),
    border: Color::srgb_u8(71, 85, 105),
    text_primary: Color::srgb_u8(248, 250, 252),
    text_secondary: Color::srgb_u8(203, 213, 225),
    accept_button: Color::srgb_u8(34, 197, 94),
    accept_button_hover: Color::srgb_u8(22, 163, 74),
    decline_button: Color::srgb_u8(71, 85, 105),
    decline_button_hover: Color::srgb_u8(51, 65, 85),
  });
}
```

### Read and update consent state manually

```rust,no_run
use bevy::prelude::*;
use bevy_mod_indigauge::prelude::*;

fn opt_out(mut consent: ResMut<IndigaugeConsentState>) {
  consent.choice = IndigaugeConsentChoice::Declined;
  consent.persist_to_disk = true;
}
```

When `persist_to_disk` is `true`, any changed choice is written on the next update tick.

## Tracing support

Send events to the Indigauge API through tracing. This is useful for debugging and monitoring your game.

### Enable the tracing feature

```toml
[dependencies]
bevy = { version = "0.19" }
bevy-mod-indigauge = { version = "0.7", features = ["tracing"] }
```

```rust,ignore
use std::time::Duration;
use bevy::{log::{LogPlugin, BoxedLayer}, prelude::*, time::common_conditions::on_timer};
use bevy_mod_indigauge::{prelude::*, tracing::{IndigaugeLayer, default_bevy_indigauge_layer}};

/// Default tracing layer, will send all events to the Indigauge API.
pub fn default_indigauge_layer(_app: &mut App) -> Option<BoxedLayer> {
  Some(Box::new(default_bevy_indigauge_layer()))
}

/// Custom tracing layer, will only send events that has an event_type, is either info, warn, or 
/// error and is not from the bevy_mod_othercrate module to the Indigauge API.
pub fn custom_indigauge_layer(_app: &mut App) -> Option<BoxedLayer> {
  Some(Box::new(
    default_bevy_indigauge_layer()
      .with_event_type_required(true) 
      .with_filters(vec!["bevy_mod_othercrate"])
      .with_levels(vec![
        IndigaugeLogLevel::Info,
        IndigaugeLogLevel::Warn,
        IndigaugeLogLevel::Error,
      ]),
  ))
}

fn main() {
  App::new()
    .add_plugins(DefaultPlugins.set(LogPlugin {custom_layer: custom_indigauge_layer, ..default()}))
    .add_plugins(IndigaugePlugin::<EmptySessionMeta>::new("YOUR_PUBLIC_KEY", "My game name", env!("CARGO_PKG_VERSION")))
    .add_systems(Startup, setup)
    .add_systems(Update, (track_counter.run_if(on_timer(Duration::from_secs(2)))))
    .run();
}

fn setup(mut commands: Commands) {
  commands.spawn((Camera2d, IsDefaultUiCamera));
  commands.trigger(StartSessionEvent::new());
}

fn track_counter(mut counter: Local<u32>) {
  *counter += 1;
  info!(ig = "counter.increase", value = *counter);
}
```

## Bevy Compatibility

| bevy   | bevy-mod-indigauge |
| ------ | ------------------ |
| 0.19   | 0.7                |
| 0.18   | 0.6                |
| 0.17   | 0.5                |
| 0.16   | 0.4                |
| 0.15   | 0.1, 0.2, 0.3      |
