use bevy::prelude::*;

use crate::feedback::components::{ButtonHoverStyle, ButtonPressedStyle};

/// Returns a bundle with default rounded panel styling.
pub fn panel(background_color: Color, border_color: Color) -> impl Bundle {
  (BorderRadius::all(Val::Px(8.0)), BackgroundColor(background_color), BorderColor(border_color))
}

/// Returns a styled Bevy button bundle used by the feedback UI.
pub fn button(background_color: Color, border_color: Color) -> impl Bundle {
  (
    Button,
    BorderRadius::all(Val::Px(8.0)),
    BackgroundColor(background_color),
    BorderColor(border_color),
    ButtonHoverStyle {
      background: background_color.with_alpha(0.5),
      border: border_color.with_alpha(0.5),
    },
    ButtonPressedStyle {
      background: background_color.with_alpha(0.2),
      border: border_color.with_alpha(0.2),
    },
  )
}
