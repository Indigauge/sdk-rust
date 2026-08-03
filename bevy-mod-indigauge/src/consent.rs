use bevy::prelude::*;

use crate::consent::{
  resources::IndigaugeConsentState,
  systems::{
    emit_consent_selected_event, enforce_consent_gate, load_persisted_consent, persist_consent_choice,
    sync_core_consent_flag,
  },
};

#[cfg(feature = "consent")]
use crate::consent::{
  resources::ConsentModalStyles,
  systems::{
    consent_modal_visibility_sync, despawn_consent_modal_ui, dismiss_modal_if_choice_known,
    observe_accept_consent_click, observe_decline_consent_click, spawn_consent_modal_ui,
  },
};

#[cfg(feature = "consent")]
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
      .add_systems(Startup, (load_persisted_consent, sync_core_consent_flag).chain())
      .add_systems(
        Update,
        (
          sync_core_consent_flag,
          enforce_consent_gate,
          persist_consent_choice,
          emit_consent_selected_event,
          #[cfg(feature = "consent")]
          dismiss_modal_if_choice_known,
        )
          .run_if(resource_changed::<IndigaugeConsentState>),
      );

    #[cfg(feature = "consent")]
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
