use std::{env, time::Duration};

use bevy::{prelude::*, time::common_conditions::on_timer};
use bevy_feathers::{FeathersPlugins, display::label};
use bevy_mod_indigauge::prelude::{
  ConsentModalProps, ConsentModalStyles, EmptySessionMeta, FeedbackCategory, FeedbackPanelProps, FeedbackPanelStyles,
  IndigaugeConsentChoice, IndigaugeConsentState, IndigaugeLogLevel, IndigaugeMode, IndigaugePlugin, StartSessionEvent,
  ig_info,
};

struct EventType;

impl EventType {
  const COUNTER_INCREASE: &'static str = "counter.increase";
}

fn main() {
  App::new()
    .add_plugins(DefaultPlugins)
    .add_plugins(FeathersPlugins)
    .add_plugins(
      IndigaugePlugin::<EmptySessionMeta>::new("YOUR_PUBLIC_KEY", "My minimal game", env!("CARGO_PKG_VERSION"))
        // Optional: Set mode (Defaults to live). Dev mode is useful for testing and debugging and does not send events to the server.
        .mode(IndigaugeMode::Dev)
        // Optional: Set preferred log-level (Defaults to Info)
        .log_level(IndigaugeLogLevel::Info),
    )
    // Optional: Customize the feedback panel styles
    .insert_resource(FeedbackPanelStyles {
      primary: Color::srgb_u8(147, 164, 255),
      primary_hover: Color::srgb_u8(124, 140, 250),
      secondary: Color::srgb_u8(147, 164, 255),
      secondary_hover: Color::srgb_u8(124, 140, 250),
      background: Color::srgb_u8(15, 23, 42),
      surface: Color::srgb_u8(30, 41, 59),
      border: Color::srgb_u8(51, 65, 85),
      text_primary: Color::srgb_u8(248, 250, 252),
      text_secondary: Color::srgb_u8(203, 213, 225),
      success: Color::srgb_u8(34, 197, 94),
      error: Color::srgb_u8(248, 113, 113),
      warning: Color::srgb_u8(250, 204, 21),
      accent: Color::srgb_u8(168, 85, 247),
    })
    .insert_resource(ConsentModalStyles {
      overlay: Color::srgba_u8(10, 15, 26, 214),
      background: Color::srgb_u8(15, 23, 42),
      border: Color::srgb_u8(71, 85, 105),
      text_primary: Color::srgb_u8(248, 250, 252),
      text_secondary: Color::srgb_u8(203, 213, 225),
      accept_button: Color::srgb_u8(34, 197, 94),
      accept_button_hover: Color::srgb_u8(22, 163, 74),
      decline_button: Color::srgb_u8(71, 85, 105),
      decline_button_hover: Color::srgb_u8(51, 65, 85),
    })
    .add_systems(Startup, setup)
    .add_systems(
      Update,
      (
        setup_consent_modal,
        trigger_session_after_persisted_accept,
        trigger_feedback_with_question,
        track_counter.run_if(on_timer(Duration::from_secs(2))),
      ),
    )
    .run();
}

fn setup(mut commands: Commands) {
  commands.spawn((Camera2d, IsDefaultUiCamera));

  const HELP_TEXT_DEFAULT: &str = "Press 'F2' to display the default feedback panel!\n";
  const HELP_TEXT_WITH_QUESTION: &str = "Press 'SPACE' to display the feedback panel with a question!\n";
  const HELP_TEXT_WITH_CONSENT: &str = "Consent modal is shown before session start.\n";

  commands.spawn_scene(bsn! {
    Node {
      flex_direction: FlexDirection::Column,
      row_gap: Val::Px(8.0),
    }
    Children [
      (label(HELP_TEXT_WITH_CONSENT)),
      (label(HELP_TEXT_DEFAULT)),
      (label(HELP_TEXT_WITH_QUESTION)),
    ]
  });
}

fn setup_consent_modal(
  mut commands: Commands,
  consent_state: Res<IndigaugeConsentState>,
  existing: Option<Res<ConsentModalProps>>,
) {
  if consent_state.choice == IndigaugeConsentChoice::Unknown && existing.is_none() {
    commands.insert_resource(
      ConsentModalProps::new()
        .title("Help improve this game")
        .message("Allow anonymous telemetry so we can improve performance, balance, and stability.")
        .accept_button_text("Allow telemetry")
        .decline_button_text("No thanks"),
    );
  }
}

fn trigger_session_after_persisted_accept(
  mut commands: Commands,
  consent_state: Res<IndigaugeConsentState>,
  mut started: Local<bool>,
) {
  if *started || !consent_state.is_accepted() {
    return;
  }

  commands.trigger(StartSessionEvent::new().with_platform("steam"));
  *started = true;
}

fn trigger_feedback_with_question(
  mut commands: Commands,
  keys: Res<ButtonInput<KeyCode>>,
  existing: Option<Res<FeedbackPanelProps>>,
) {
  if existing.is_some() {
    return;
  }

  if keys.just_pressed(KeyCode::Space) {
    commands.insert_resource(FeedbackPanelProps::with_question(
      "What did you think about level 3?",
      FeedbackCategory::Gameplay,
    ));
  }
}

fn track_counter(mut counter: Local<u32>) {
  *counter += 1;
  ig_info!(EventType::COUNTER_INCREASE, { "value": *counter });
}
