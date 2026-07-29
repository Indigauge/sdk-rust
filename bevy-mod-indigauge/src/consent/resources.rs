use bevy::prelude::*;

/// The current telemetry consent choice selected by the player.
#[derive(Debug, Default, Clone, Copy, PartialEq, Eq)]
pub enum IndigaugeConsentChoice {
  Accepted,
  Declined,
  #[default]
  Unknown,
}

impl IndigaugeConsentChoice {
  pub(crate) fn as_disk_value(self) -> Option<&'static str> {
    match self {
      Self::Accepted => Some("accepted"),
      Self::Declined => Some("declined"),
      Self::Unknown => None,
    }
  }

  pub(crate) fn from_disk_value(value: &str) -> Option<Self> {
    match value.trim().to_ascii_lowercase().as_str() {
      "accepted" => Some(Self::Accepted),
      "declined" => Some(Self::Declined),
      _ => None,
    }
  }
}

/// Resource storing the player's telemetry consent state.
#[derive(Resource, Debug, Clone, Copy)]
pub struct IndigaugeConsentState {
  pub choice: IndigaugeConsentChoice,
  pub persist_to_disk: bool,
}

impl Default for IndigaugeConsentState {
  fn default() -> Self {
    Self {
      choice: IndigaugeConsentChoice::Unknown,
      persist_to_disk: true,
    }
  }
}

impl IndigaugeConsentState {
  /// Returns true when the user has accepted telemetry consent.
  pub fn is_accepted(&self) -> bool {
    self.choice == IndigaugeConsentChoice::Accepted
  }

  /// Returns true when the user has declined telemetry consent.
  pub fn is_declined(&self) -> bool {
    self.choice == IndigaugeConsentChoice::Declined
  }
}

/// Screen placement options for spawning the consent modal.
#[cfg(feature = "feedback")]
#[derive(Default, Debug, Clone, Copy)]
pub enum ConsentModalSpawnPosition {
  TopLeft,
  TopRight,
  TopCenter,
  BottomLeft,
  BottomRight,
  BottomCenter,
  #[default]
  Center,
  CenterLeft,
  CenterRight,
}

#[cfg(feature = "feedback")]
impl ConsentModalSpawnPosition {
  pub(crate) fn align_items(&self) -> AlignItems {
    match self {
      ConsentModalSpawnPosition::TopLeft
      | ConsentModalSpawnPosition::TopCenter
      | ConsentModalSpawnPosition::TopRight => AlignItems::Start,
      ConsentModalSpawnPosition::BottomLeft
      | ConsentModalSpawnPosition::BottomCenter
      | ConsentModalSpawnPosition::BottomRight => AlignItems::End,
      _ => AlignItems::Center,
    }
  }

  pub(crate) fn justify_content(&self) -> JustifyContent {
    match self {
      ConsentModalSpawnPosition::TopLeft
      | ConsentModalSpawnPosition::BottomLeft
      | ConsentModalSpawnPosition::CenterLeft => JustifyContent::Start,
      ConsentModalSpawnPosition::TopRight
      | ConsentModalSpawnPosition::BottomRight
      | ConsentModalSpawnPosition::CenterRight => JustifyContent::End,
      _ => JustifyContent::Center,
    }
  }
}

/// Style palette used by the consent modal.
#[cfg(feature = "feedback")]
#[derive(Resource, Debug)]
pub struct ConsentModalStyles {
  pub overlay: Color,
  pub background: Color,
  pub border: Color,
  pub text_primary: Color,
  pub text_secondary: Color,
  pub accept_button: Color,
  pub accept_button_hover: Color,
  pub decline_button: Color,
  pub decline_button_hover: Color,
}

#[cfg(feature = "feedback")]
impl Default for ConsentModalStyles {
  fn default() -> Self {
    Self {
      overlay: Color::srgba_u8(2, 6, 23, 192),
      background: Color::srgb_u8(15, 23, 42),
      border: Color::srgb_u8(51, 65, 85),
      text_primary: Color::srgb_u8(248, 250, 252),
      text_secondary: Color::srgb_u8(203, 213, 225),
      accept_button: Color::srgb_u8(34, 197, 94),
      accept_button_hover: Color::srgb_u8(22, 163, 74),
      decline_button: Color::srgb_u8(71, 85, 105),
      decline_button_hover: Color::srgb_u8(51, 65, 85),
    }
  }
}

/// Controls how and when the telemetry consent modal is shown.
#[cfg(feature = "feedback")]
#[derive(Resource, Debug)]
pub struct ConsentModalProps {
  pub title: String,
  pub message: String,
  pub accept_button_text: String,
  pub decline_button_text: String,
  pub visible: bool,
  pub persist_choice: bool,
  pub spawn_position: ConsentModalSpawnPosition,
  pub position_margin: UiRect,
}

#[cfg(feature = "feedback")]
impl Default for ConsentModalProps {
  fn default() -> Self {
    Self {
      title: "Share anonymous gameplay data?".to_string(),
      message: "We use gameplay telemetry and diagnostics to improve game balance, stability, and performance. You can change this later in your game settings.".to_string(),
      accept_button_text: "Allow".to_string(),
      decline_button_text: "Decline".to_string(),
      visible: true,
      persist_choice: true,
      spawn_position: ConsentModalSpawnPosition::default(),
      position_margin: UiRect::all(px(0)),
    }
  }
}

#[cfg(feature = "feedback")]
impl ConsentModalProps {
  /// Creates consent modal props with SDK defaults.
  pub fn new() -> Self {
    Self::default()
  }

  /// Sets a custom title for the consent modal.
  pub fn title(mut self, title: impl Into<String>) -> Self {
    self.title = title.into();
    self
  }

  /// Sets custom body text for the consent modal.
  pub fn message(mut self, message: impl Into<String>) -> Self {
    self.message = message.into();
    self
  }

  /// Sets custom text for the accept/allow button.
  pub fn accept_button_text(mut self, text: impl Into<String>) -> Self {
    self.accept_button_text = text.into();
    self
  }

  /// Sets custom text for the decline button.
  pub fn decline_button_text(mut self, text: impl Into<String>) -> Self {
    self.decline_button_text = text.into();
    self
  }

  /// Sets whether this modal should currently be visible.
  pub fn visible(mut self, visible: bool) -> Self {
    self.visible = visible;
    self
  }

  /// Sets whether the selected choice should be persisted on disk.
  pub fn persist_choice(mut self, persist_choice: bool) -> Self {
    self.persist_choice = persist_choice;
    self
  }

  /// Sets where the modal should appear on screen.
  pub fn spawn_position(mut self, spawn_position: ConsentModalSpawnPosition) -> Self {
    self.spawn_position = spawn_position;
    self
  }

  /// Sets outer margin around the modal root.
  pub fn margin(mut self, margin: UiRect) -> Self {
    self.position_margin = margin;
    self
  }
}
