use bevy::prelude::*;
use bevy::scene::Scene;
use bevy::{picking::hover::Hovered, ui_widgets::Button};

use crate::feedback::components::{ButtonHoverStyle, ButtonPressedStyle};

/// Returns a scene with the feedback panel colors.
pub fn panel(background_color: Color, border_color: Color) -> impl Scene {
  bsn! {
    BackgroundColor(background_color)
    BorderColor::all(border_color)
  }
}

/// Returns a styled Bevy button scene used by the feedback UI.
pub fn button(background_color: Color, border_color: Color) -> impl Scene {
  bsn! {
    Button
    Hovered::default()
    @panel(background_color, border_color)
    ButtonHoverStyle {
      background: {background_color.with_alpha(0.5)},
      border: {border_color.with_alpha(0.5)},
    }
    ButtonPressedStyle {
      background: {background_color.with_alpha(0.2)},
      border: {border_color.with_alpha(0.2)},
    }
  }
}
