use bevy::prelude::*;

#[cfg(all(feature = "feedback", not(feature = "feedback_egui")))]
use bevy_text_edit::TextEditPluginAnyState;

use crate::{feedback::resources::*, session::resources::SessionApiKey};

pub mod components;
#[cfg(feature = "feedback_egui")]
mod egui;
#[cfg(all(feature = "feedback", not(feature = "feedback_egui")))]
pub mod helpers;
pub(crate) mod observers;
pub mod resources;
#[cfg(all(feature = "feedback", not(feature = "feedback_egui")))]
mod systems;
pub mod types;

/// Plugin that exposes in-game feedback UI and submission flow.
pub struct FeedbackUiPlugin;

impl Plugin for FeedbackUiPlugin {
  fn build(&self, app: &mut App) {
    app
      .init_resource::<FeedbackFormState>()
      .init_resource::<FeedbackKeyCodeToggle>()
      .init_resource::<FeedbackPanelStyles>();

    #[cfg(all(feature = "feedback", not(feature = "feedback_egui")))]
    app
      .add_plugins(TextEditPluginAnyState::any())
      .insert_resource(FeedbackUiState::default())
      .add_systems(
        Update,
        (
          systems::spawn_feedback_ui.run_if(resource_exists_and_changed::<FeedbackPanelProps>),
          systems::despawn_feedback_panel.run_if(resource_removed::<FeedbackPanelProps>),
          systems::toggle_panel_visibility_with_key.run_if(resource_exists::<FeedbackKeyCodeToggle>),
          systems::panel_visibility_sync.run_if(resource_exists_and_changed::<FeedbackPanelProps>),
          systems::dropdown_visibility_sync.run_if(resource_exists_and_changed::<FeedbackFormState>),
          systems::update_scroll_position,
          systems::handle_hover_and_click_styles,
        )
          .run_if(resource_exists::<SessionApiKey>),
      );

    #[cfg(feature = "feedback_egui")]
    {
      egui::ensure_egui_plugin(app);

      app.add_systems(
        Update,
        (
          egui::spawn_feedback_marker.run_if(resource_exists_and_changed::<FeedbackPanelProps>),
          egui::despawn_feedback_marker.run_if(resource_removed::<FeedbackPanelProps>),
          egui::toggle_panel_visibility_with_key.run_if(resource_exists::<FeedbackKeyCodeToggle>),
          egui::draw_feedback_ui.run_if(resource_exists::<FeedbackPanelProps>),
        )
          .run_if(resource_exists::<SessionApiKey>),
      );
    }
  }
}
