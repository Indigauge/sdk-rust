use bevy::prelude::*;

#[cfg(all(feature = "feedback", not(feature = "feedback_egui")))]
use crate::feedback::types::FeedbackCategory;

#[cfg(all(feature = "feedback", not(feature = "feedback_egui")))]
#[derive(Component)]
/// Stores original button colors for state restoration.
pub struct OriginalButtonStyles {
  pub background: Color,
  pub border: Color,
}

#[cfg(all(feature = "feedback", not(feature = "feedback_egui")))]
#[derive(Component)]
/// Hover-state style colors for button widgets.
pub struct ButtonHoverStyle {
  pub background: Color,
  pub border: Color,
}

#[cfg(all(feature = "feedback", not(feature = "feedback_egui")))]
#[derive(Component)]
/// Pressed-state style colors for button widgets.
pub struct ButtonPressedStyle {
  pub background: Color,
  pub border: Color,
}

#[derive(Component)]
/// Marker component for the root feedback panel entity.
pub struct FeedbackPanel;

#[cfg(all(feature = "feedback", not(feature = "feedback_egui")))]
#[derive(Component)]
/// Marker for the free-text feedback input field.
pub struct MessageInput;

#[cfg(all(feature = "feedback", not(feature = "feedback_egui")))]
#[derive(Component)]
/// Marker for the root container of the message input.
pub struct MessageInputRoot;

#[cfg(all(feature = "feedback", not(feature = "feedback_egui")))]
#[derive(Component)]
/// Marker for the question text root entity.
pub struct QuestionTextRoot;

// #[derive(Component)]
// pub struct RatingStar(pub u8);

#[cfg(all(feature = "feedback", not(feature = "feedback_egui")))]
#[derive(Component)]
/// Marker for the feedback category dropdown button.
pub struct CategoryButton;

#[cfg(all(feature = "feedback", not(feature = "feedback_egui")))]
#[derive(Component)]
/// Marker for the feedback category list container.
pub struct CategoryList; // container som toggles

#[cfg(all(feature = "feedback", not(feature = "feedback_egui")))]
#[derive(Component)]
/// Marker for a selectable feedback category item.
pub struct CategoryItem(pub FeedbackCategory);

#[cfg(all(feature = "feedback", not(feature = "feedback_egui")))]
#[derive(Component)]
/// Marker for the feedback submit button.
pub struct SubmitButton;

#[cfg(all(feature = "feedback", not(feature = "feedback_egui")))]
#[derive(Component)]
/// Marker for the feedback cancel button.
pub struct CancelButton;

#[cfg(all(feature = "feedback", not(feature = "feedback_egui")))]
#[derive(Component)]
#[require(HoldPressed)]
/// Marker for the include-screenshot toggle control.
pub struct ScreenshotToggle;

#[cfg(all(feature = "feedback", not(feature = "feedback_egui")))]
#[derive(Component)]
/// Marker for screenshot toggle text.
pub struct ScreenshotToggleText;

#[cfg(all(feature = "feedback", not(feature = "feedback_egui")))]
#[derive(Component)]
/// Marker for category button text.
pub struct CategoryButtonText;

#[cfg(all(feature = "feedback", not(feature = "feedback_egui")))]
#[derive(Component, Default)]
/// Marker indicating the pressed state should be held after click.
pub struct HoldPressed;

#[cfg(all(feature = "feedback", not(feature = "feedback_egui")))]
#[derive(Component)]
/// Marker indicating a toggle-like component is active.
pub struct Active;
