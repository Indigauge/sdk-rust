use bevy::prelude::*;

/// Canonical categories available for user feedback submissions.
#[derive(Clone, Copy, PartialEq, Eq, Debug, Hash, Default)]
pub enum FeedbackCategory {
  #[default]
  General,
  Ui,
  Gameplay,
  Performance,
  Bugs,
  Controls,
  Audio,
  Balance,
  Graphics,
  Visual,
  Art,
  Other,
}

impl FeedbackCategory {
  /// All categories in display order.
  pub const ALL: &'static [FeedbackCategory] = &[
    FeedbackCategory::General,
    FeedbackCategory::Ui,
    FeedbackCategory::Gameplay,
    FeedbackCategory::Performance,
    FeedbackCategory::Bugs,
    FeedbackCategory::Controls,
    FeedbackCategory::Audio,
    FeedbackCategory::Balance,
    FeedbackCategory::Graphics,
    FeedbackCategory::Visual,
    FeedbackCategory::Art,
    FeedbackCategory::Other,
  ];

  /// Human-readable label for this category.
  pub fn label(&self) -> &'static str {
    match self {
      FeedbackCategory::General => "General",
      FeedbackCategory::Ui => "UI",
      FeedbackCategory::Gameplay => "Gameplay",
      FeedbackCategory::Performance => "Performance",
      FeedbackCategory::Bugs => "Bugs",
      FeedbackCategory::Other => "Other",
      FeedbackCategory::Controls => "Controls",
      FeedbackCategory::Audio => "Audio",
      FeedbackCategory::Balance => "Balance",
      FeedbackCategory::Graphics => "Graphics",
      FeedbackCategory::Visual => "Visual",
      FeedbackCategory::Art => "Art",
    }
  }
}

/// Screen placement options for spawning the feedback panel.
#[derive(Default)]
pub enum FeedbackSpawnPosition {
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

impl FeedbackSpawnPosition {
  /// Returns the panel vertical alignment for this spawn position.
  pub fn align_items(&self) -> AlignItems {
    match self {
      FeedbackSpawnPosition::TopLeft | FeedbackSpawnPosition::TopCenter | FeedbackSpawnPosition::TopRight => {
        AlignItems::Start
      },
      FeedbackSpawnPosition::BottomLeft | FeedbackSpawnPosition::BottomCenter | FeedbackSpawnPosition::BottomRight => {
        AlignItems::End
      },
      _ => AlignItems::Center,
    }
  }

  /// Returns the panel horizontal alignment for this spawn position.
  pub fn justify_content(&self) -> JustifyContent {
    match self {
      FeedbackSpawnPosition::TopLeft | FeedbackSpawnPosition::BottomLeft | FeedbackSpawnPosition::CenterLeft => {
        JustifyContent::Start
      },
      FeedbackSpawnPosition::TopRight | FeedbackSpawnPosition::BottomRight | FeedbackSpawnPosition::CenterRight => {
        JustifyContent::End
      },
      _ => JustifyContent::Center,
    }
  }
}
