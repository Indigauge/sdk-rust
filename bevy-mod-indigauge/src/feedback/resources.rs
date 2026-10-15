use std::time::Duration;

use bevy::prelude::*;

use crate::feedback::types::{FeedbackCategory, FeedbackSpawnPosition};

/// Key binding resource used to toggle feedback panel visibility.
#[derive(Resource, Debug)]
pub struct FeedbackKeyCodeToggle(pub KeyCode);

impl Default for FeedbackKeyCodeToggle {
  fn default() -> Self {
    Self(KeyCode::F2)
  }
}

/// Style palette resource used when rendering the feedback UI.
#[derive(Resource, Debug)]
pub struct FeedbackPanelStyles {
  pub primary: Color,
  pub primary_hover: Color,
  pub secondary: Color,
  pub secondary_hover: Color,
  pub background: Color,
  pub surface: Color,
  pub border: Color,
  pub text_primary: Color,
  pub text_secondary: Color,
  pub success: Color,
  pub error: Color,
  pub warning: Color,
  pub accent: Color,
}

impl Default for FeedbackPanelStyles {
  fn default() -> Self {
    Self {
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
    }
  }
}

/// A [`Resource`] that controls the visibility and behavior of the in-game
/// feedback panel.
///
/// This resource is inserted into the [`World`](bevy::prelude::World) when you
/// want to show the feedback form manually. The [`IndigaugePlugin`](crate::prelude::IndigaugePlugin) listens
/// for it and renders a user interface that allows players to submit feedback
/// directly from within the game.
///
/// The feedback panel can optionally display a custom title and question, and
/// categorize the submitted feedback (for example, “Gameplay”, “UI”, or
/// “Performance”).
///
/// # Usage
///
/// You can insert this resource into the world at any time to display a feedback
/// form. For example:
///
/// ```rust
/// use bevy::prelude::*;
/// use bevy_mod_indigauge::prelude::*;
///
/// fn trigger_feedback_with_question(
///     mut commands: Commands,
///     keys: Res<ButtonInput<KeyCode>>,
/// ) {
///     if keys.just_pressed(KeyCode::Space) {
///         // Manually trigger the feedback panel with a predefined question.
///         commands.insert_resource(
///             FeedbackPanelProps::with_question(
///                 "What did you think about level 3?",
///                 FeedbackCategory::Gameplay,
///             ),
///         );
///     }
/// }
/// ```
///
/// # Fields
///
/// * `title` – Optional custom title shown at the top of the feedback form.
/// * `question` – Optional question to ask the player. Useful when you want
///   feedback about a specific moment or feature.
/// * `category` – Optional [`FeedbackCategory`] to tag the feedback (e.g.,
///   `Gameplay`, `UI`, `Bug`, etc.). If not provided, a category dropdown will be shown.
/// * `visible` – Whether the feedback panel is currently visible.
///   Managed automatically by the plugin.
/// * `allow_screenshot` – Whether the player should be allowed to attach
///   a screenshot with their feedback submission.
/// * `show_thank_you` - Whether to display a thank-you message after success (default: true).
/// * `thank_you_duration` - Time before closing the success message (default: two seconds).
/// * `thank_you_message` - Text displayed after success (default: "Thank you for your feedback!").
///
/// # See Also
///
/// * [`FeedbackCategory`] – Defines available categories for feedback.
///
#[derive(Resource)]
pub struct FeedbackPanelProps {
  /// Optional custom title shown at the top of the feedback form.
  pub(crate) title: Option<String>,

  /// Optional question to ask the player.
  pub(crate) question: Option<String>,

  /// The category to tag the feedback with (e.g., Gameplay, UI, Bug).
  pub(crate) category: Option<FeedbackCategory>,

  /// Whether the feedback panel is currently visible.
  pub visible: bool,

  /// Whether screenshots are allowed when submitting feedback.
  pub(crate) allow_screenshot: bool,

  pub(crate) spawn_position: FeedbackSpawnPosition,

  pub(crate) position_margin: UiRect,

  /// Whether to show a thank-you message after successful submission. Defaults to true.
  pub(crate) show_thank_you: bool,

  /// How long the thank-you message is displayed before closing. Defaults to two seconds.
  pub(crate) thank_you_duration: Duration,

  /// Text displayed after successful submission.
  pub(crate) thank_you_message: String,
}

impl Default for FeedbackPanelProps {
  /// Creates a new feedback panel with "Send feedback" as title, no question and no category.
  /// Screenshots are allowed by default.
  fn default() -> Self {
    Self {
      title: Some("Send feedback".to_string()),
      question: None,
      category: None,
      visible: true,
      allow_screenshot: true,
      spawn_position: FeedbackSpawnPosition::default(),
      position_margin: UiRect::all(Val::Px(16.0)),
      show_thank_you: true,
      thank_you_duration: Duration::from_secs(2),
      thank_you_message: "Thank you for your feedback!".to_string(),
    }
  }
}

impl FeedbackPanelProps {
  /// Creates a new feedback panel with a question and a fixed category.
  /// Screenshots are not allowed by default.
  pub fn with_question(question: impl Into<String>, category: FeedbackCategory) -> Self {
    Self {
      title: None,
      question: Some(question.into()),
      category: Some(category),
      visible: true,
      allow_screenshot: false,
      ..Default::default()
    }
  }

  /// Set a custom title for the feedback panel
  pub fn title(mut self, title: impl Into<String>) -> Self {
    self.title = Some(title.into());
    self
  }

  /// Sets whether screenshot attachments are allowed.
  pub fn allow_screenshot(mut self, allow_screenshot: bool) -> Self {
    self.allow_screenshot = allow_screenshot;
    self
  }

  /// Sets where the feedback panel should appear on screen.
  pub fn spawn_position(mut self, spawn_position: FeedbackSpawnPosition) -> Self {
    self.spawn_position = spawn_position;
    self
  }

  /// Sets outer margin around the feedback panel.
  pub fn margin(mut self, margin: UiRect) -> Self {
    self.position_margin = margin;
    self
  }

  /// Enables or disables the thank-you message after successful submission.
  pub fn show_thank_you(mut self, show_thank_you: bool) -> Self {
    self.show_thank_you = show_thank_you;
    self
  }

  /// Sets how long the thank-you message is displayed before the panel closes.
  pub fn thank_you_duration(mut self, duration: Duration) -> Self {
    self.thank_you_duration = duration;
    self
  }

  /// Sets the thank-you text displayed after successful submission.
  pub fn thank_you_message(mut self, message: impl Into<String>) -> Self {
    self.thank_you_message = message.into();
    self
  }
}

/// Mutable state used while the user fills out the feedback form.
#[derive(Resource, Default)]
pub struct FeedbackFormState {
  // pub rating: u8,                 // 1..=5
  pub category: FeedbackCategory, // dropdown-valg
  pub message: String,
  pub include_screenshot: bool,
  pub question: Option<String>,
  pub error: Option<String>,
  pub(crate) pending_submission: Option<Entity>,
  pub(crate) thank_you_remaining: Option<Duration>,
}

impl FeedbackFormState {
  pub(crate) fn complete_submission(&mut self, props: &FeedbackPanelProps) {
    self.pending_submission = None;
    self.error = None;
    self.thank_you_remaining = props.show_thank_you.then_some(props.thank_you_duration);
  }
}

// UI-specific transient state for the legacy Bevy UI backend
#[cfg(all(feature = "feedback", not(feature = "feedback_egui")))]
#[derive(Resource, Default)]
/// UI-only transient state for dropdown behavior.
pub struct FeedbackUiState {
  pub dropdown_open: bool,
}

/// Marker resource indicating that next feedback submission should include screenshot capture.
#[derive(Resource)]
pub struct TakeScreenshot;

#[cfg(test)]
mod tests {
  use super::*;

  #[test]
  fn thank_you_defaults_and_configuration() {
    let props = FeedbackPanelProps::default();
    assert!(props.show_thank_you);
    assert_eq!(props.thank_you_duration, Duration::from_secs(2));
    assert_eq!(props.thank_you_message, "Thank you for your feedback!");

    let props = FeedbackPanelProps::with_question("What went wrong?", FeedbackCategory::Bugs)
      .show_thank_you(false)
      .thank_you_duration(Duration::from_millis(500))
      .thank_you_message("Thanks for helping us improve!");
    assert!(!props.show_thank_you);
    assert_eq!(props.thank_you_duration, Duration::from_millis(500));
    assert_eq!(props.thank_you_message, "Thanks for helping us improve!");
  }

  #[test]
  fn successful_submission_starts_configured_thank_you() {
    let mut form = FeedbackFormState {
      pending_submission: Some(Entity::PLACEHOLDER),
      error: Some("Previous error".to_string()),
      ..default()
    };
    form.complete_submission(&FeedbackPanelProps::default().thank_you_duration(Duration::from_secs(3)));
    assert!(form.pending_submission.is_none());
    assert!(form.error.is_none());
    assert_eq!(form.thank_you_remaining, Some(Duration::from_secs(3)));

    form.complete_submission(&FeedbackPanelProps::default().show_thank_you(false));
    assert!(form.thank_you_remaining.is_none());
  }
}
