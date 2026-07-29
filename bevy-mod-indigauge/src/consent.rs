use bevy::prelude::*;

use crate::consent::{
  resources::IndigaugeConsentState,
  systems::{load_persisted_consent, persist_consent_choice},
};

#[cfg(feature = "feedback")]
use crate::consent::{
  resources::ConsentModalStyles,
  systems::{
    consent_modal_visibility_sync, despawn_consent_modal_ui, observe_accept_consent_click,
    observe_decline_consent_click, spawn_consent_modal_ui,
  },
};

#[cfg(feature = "feedback")]
pub mod components;
pub mod events;
pub mod resources;
pub(crate) mod systems;
pub(crate) mod utils;

/// Plugin that manages telemetry consent state and consent modal UI.
pub struct ConsentPlugin;

impl Plugin for ConsentPlugin {
  fn build(&self, app: &mut App) {
    app
      .init_resource::<IndigaugeConsentState>()
      .add_systems(Startup, load_persisted_consent)
      .add_systems(Update, persist_consent_choice.run_if(resource_changed::<IndigaugeConsentState>));

    #[cfg(feature = "feedback")]
    app.init_resource::<ConsentModalStyles>().add_systems(
      Update,
      (
        spawn_consent_modal_ui.run_if(resource_exists_and_changed::<resources::ConsentModalProps>),
        despawn_consent_modal_ui.run_if(resource_removed::<resources::ConsentModalProps>),
        consent_modal_visibility_sync.run_if(resource_exists_and_changed::<resources::ConsentModalProps>),
        observe_accept_consent_click.run_if(resource_exists::<resources::ConsentModalProps>),
        observe_decline_consent_click.run_if(resource_exists::<resources::ConsentModalProps>),
      ),
    );
  }
}
