use bevy::prelude::*;

#[cfg(all(feature = "feedback", not(feature = "feedback_egui")))]
use crate::feedback::types::FeedbackCategory;

#[cfg(all(feature = "feedback", not(feature = "feedback_egui")))]
#[derive(Component)]
pub struct OriginalButtonStyles {
  pub background: Color,
  pub border: Color,
}

#[cfg(all(feature = "feedback", not(feature = "feedback_egui")))]
#[derive(Component)]
pub struct ButtonHoverStyle {
  pub background: Color,
  pub border: Color,
}

#[cfg(all(feature = "feedback", not(feature = "feedback_egui")))]
#[derive(Component)]
pub struct ButtonPressedStyle {
  pub background: Color,
  pub border: Color,
}

#[derive(Component)]
pub struct FeedbackPanel;

#[cfg(all(feature = "feedback", not(feature = "feedback_egui")))]
#[derive(Component)]
pub struct MessageInput;

#[cfg(all(feature = "feedback", not(feature = "feedback_egui")))]
#[derive(Component)]
pub struct QuestionTextRoot;

// #[derive(Component)]
// pub struct RatingStar(pub u8);

#[cfg(all(feature = "feedback", not(feature = "feedback_egui")))]
#[derive(Component)]
pub struct CategoryButton;

#[cfg(all(feature = "feedback", not(feature = "feedback_egui")))]
#[derive(Component)]
pub struct CategoryList; // container som toggles

#[cfg(all(feature = "feedback", not(feature = "feedback_egui")))]
#[derive(Component)]
pub struct CategoryItem(pub FeedbackCategory);

#[cfg(all(feature = "feedback", not(feature = "feedback_egui")))]
#[derive(Component)]
pub struct SubmitButton;

#[cfg(all(feature = "feedback", not(feature = "feedback_egui")))]
#[derive(Component)]
pub struct CancelButton;

#[cfg(all(feature = "feedback", not(feature = "feedback_egui")))]
#[derive(Component)]
#[require(HoldPressed)]
pub struct ScreenshotToggle;

#[cfg(all(feature = "feedback", not(feature = "feedback_egui")))]
#[derive(Component)]
pub struct ScreenshotToggleText;

#[cfg(all(feature = "feedback", not(feature = "feedback_egui")))]
#[derive(Component)]
pub struct CategoryButtonText;

#[cfg(all(feature = "feedback", not(feature = "feedback_egui")))]
#[derive(Component, Default)]
pub struct HoldPressed;

#[cfg(all(feature = "feedback", not(feature = "feedback_egui")))]
#[derive(Component)]
pub struct Active;
