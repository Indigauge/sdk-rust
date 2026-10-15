use bevy::prelude::*;
#[cfg(all(feature = "feedback", not(feature = "feedback_egui")))]
use bevy_feathers::FeathersPlugins;

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

    app.add_systems(Update, close_after_thank_you.run_if(resource_exists::<FeedbackPanelProps>));

    #[cfg(all(feature = "feedback", not(feature = "feedback_egui")))]
    app
      .add_plugins(FeathersPlugins)
      .insert_resource(FeedbackUiState::default())
      .add_systems(
        Update,
        (
          systems::spawn_feedback_ui.run_if(resource_exists_and_changed::<FeedbackPanelProps>),
          systems::despawn_feedback_panel.run_if(resource_removed::<FeedbackPanelProps>),
          systems::toggle_panel_visibility_with_key.run_if(resource_exists::<FeedbackKeyCodeToggle>),
          systems::panel_visibility_sync.run_if(resource_exists_and_changed::<FeedbackPanelProps>),
          systems::dropdown_visibility_sync.run_if(resource_exists_and_changed::<FeedbackUiState>),
          systems::update_scroll_position,
          systems::handle_hover_and_click_styles,
          systems::error_sync.run_if(resource_exists::<FeedbackPanelProps>),
          systems::show_thank_you.run_if(resource_exists::<FeedbackPanelProps>),
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

fn close_after_thank_you(
  mut commands: Commands,
  time: Res<Time<Real>>,
  mut form: ResMut<FeedbackFormState>,
  props: Res<FeedbackPanelProps>,
) {
  if props.is_changed() {
    return;
  }
  if let Some(remaining) = &mut form.thank_you_remaining {
    *remaining = remaining.saturating_sub(time.delta());
    if remaining.is_zero() {
      commands.remove_resource::<FeedbackPanelProps>();
      form.thank_you_remaining = None;
    }
  }
}

#[cfg(test)]
mod tests {
  use super::*;
  use std::time::Duration;

  #[test]
  fn thank_you_closes_after_real_time_duration() {
    let mut app = App::new();
    app
      .init_resource::<Time<Real>>()
      .init_resource::<FeedbackFormState>()
      .insert_resource(FeedbackPanelProps::default())
      .add_systems(Update, close_after_thank_you.run_if(resource_exists::<FeedbackPanelProps>));
    app.update();
    app
      .world_mut()
      .resource_mut::<FeedbackFormState>()
      .complete_submission(&FeedbackPanelProps::default());
    app
      .world_mut()
      .resource_mut::<Time<Real>>()
      .advance_by(Duration::from_millis(1999));
    app.update();
    assert!(app.world().contains_resource::<FeedbackPanelProps>());
    app
      .world_mut()
      .resource_mut::<Time<Real>>()
      .advance_by(Duration::from_millis(1));
    app.update();
    assert!(!app.world().contains_resource::<FeedbackPanelProps>());
  }

  #[test]
  fn replacement_panel_is_not_closed_by_previous_timer() {
    let mut app = App::new();
    app
      .init_resource::<Time<Real>>()
      .init_resource::<FeedbackFormState>()
      .insert_resource(FeedbackPanelProps::default())
      .add_systems(Update, close_after_thank_you.run_if(resource_exists::<FeedbackPanelProps>));
    app.update();
    app
      .world_mut()
      .resource_mut::<FeedbackFormState>()
      .complete_submission(&FeedbackPanelProps::default());
    app.world_mut().insert_resource(FeedbackPanelProps::default());
    app
      .world_mut()
      .resource_mut::<Time<Real>>()
      .advance_by(Duration::from_secs(5));
    app.update();
    assert!(app.world().contains_resource::<FeedbackPanelProps>());
  }
}
