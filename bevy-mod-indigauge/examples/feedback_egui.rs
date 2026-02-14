use bevy::prelude::*;
use bevy_mod_indigauge::prelude::{
  EmptySessionMeta, FeedbackCategory, FeedbackPanelProps, FeedbackPanelStyles, IndigaugeLogLevel, IndigaugeMode,
  IndigaugePlugin, StartSessionEvent,
};

fn main() {
  App::new()
    .add_plugins(DefaultPlugins)
    .add_plugins(
      IndigaugePlugin::<EmptySessionMeta>::new("YOUR_PUBLIC_KEY", "Feedback Egui Example", env!("CARGO_PKG_VERSION"))
        .mode(IndigaugeMode::Dev)
        .log_level(IndigaugeLogLevel::Info),
    )
    .insert_resource(FeedbackPanelStyles::default())
    .add_systems(Startup, setup)
    .add_systems(Update, (trigger_bug_report_feedback, trigger_level_feedback))
    .run();
}

fn setup(mut commands: Commands) {
  commands.spawn((Camera2d, IsDefaultUiCamera));
  commands.trigger(StartSessionEvent::new());

  commands
    .spawn(Node {
      width: Val::Percent(100.0),
      height: Val::Percent(100.0),
      justify_content: JustifyContent::Center,
      align_items: AlignItems::Center,
      ..default()
    })
    .with_children(|root| {
      root
        .spawn(Node {
          flex_direction: FlexDirection::Column,
          row_gap: Val::Px(8.0),
          ..default()
        })
        .with_children(|column| {
          column.spawn(Text::new("bevy_egui feedback panel example"));
          column.spawn(Text::new("Press F2 for default panel"));
          column.spawn(Text::new("Press F3 for a bug report question panel"));
          column.spawn(Text::new("Press SPACE for a gameplay question panel"));
        });
    });
}

fn trigger_bug_report_feedback(
  mut commands: Commands,
  keys: Res<ButtonInput<KeyCode>>,
  existing: Option<Res<FeedbackPanelProps>>,
) {
  if existing.is_some() {
    return;
  }

  if keys.just_pressed(KeyCode::F3) {
    commands.insert_resource(
      FeedbackPanelProps::with_question("What went wrong?", FeedbackCategory::Bugs)
        .title("Bug Report")
        .allow_screenshot(true),
    );
  }
}

fn trigger_level_feedback(
  mut commands: Commands,
  keys: Res<ButtonInput<KeyCode>>,
  existing: Option<Res<FeedbackPanelProps>>,
) {
  if existing.is_some() {
    return;
  }

  if keys.just_pressed(KeyCode::Space) {
    commands.insert_resource(
      FeedbackPanelProps::with_question("What did you think about this sequence?", FeedbackCategory::Gameplay)
        .title("Level Feedback")
        .allow_screenshot(false),
    );
  }
}
