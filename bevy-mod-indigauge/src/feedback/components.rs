use bevy::prelude::*;

#[cfg(feature = "feedback_ui")]
use crate::feedback::types::FeedbackCategory;

#[cfg(feature = "feedback_ui")]
#[derive(Component)]
pub struct OriginalButtonStyles {
  pub background: Color,
  pub border: Color,
}

#[cfg(feature = "feedback_ui")]
#[derive(Component)]
pub struct ButtonHoverStyle {
  pub background: Color,
  pub border: Color,
}

#[cfg(feature = "feedback_ui")]
#[derive(Component)]
pub struct ButtonPressedStyle {
  pub background: Color,
  pub border: Color,
}

#[derive(Component)]
pub struct FeedbackPanel;

#[cfg(feature = "feedback_ui")]
#[derive(Component)]
pub struct MessageInput;

#[cfg(feature = "feedback_ui")]
#[derive(Component)]
pub struct QuestionTextRoot;

// #[derive(Component)]
// pub struct RatingStar(pub u8);

#[cfg(feature = "feedback_ui")]
#[derive(Component)]
pub struct CategoryButton;

#[cfg(feature = "feedback_ui")]
#[derive(Component)]
pub struct CategoryList; // container som toggles

#[cfg(feature = "feedback_ui")]
#[derive(Component)]
pub struct CategoryItem(pub FeedbackCategory);

#[cfg(feature = "feedback_ui")]
#[derive(Component)]
pub struct SubmitButton;

#[cfg(feature = "feedback_ui")]
#[derive(Component)]
pub struct CancelButton;

#[cfg(feature = "feedback_ui")]
#[derive(Component)]
#[require(HoldPressed)]
pub struct ScreenshotToggle;

#[cfg(feature = "feedback_ui")]
#[derive(Component)]
pub struct ScreenshotToggleText;

#[cfg(feature = "feedback_ui")]
#[derive(Component)]
pub struct CategoryButtonText;

#[cfg(feature = "feedback_ui")]
#[derive(Component, Default)]
pub struct HoldPressed;

#[cfg(feature = "feedback_ui")]
#[derive(Component)]
pub struct Active;
