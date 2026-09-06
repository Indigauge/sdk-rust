use bevy::prelude::*;

use crate::consent::resources::IndigaugeConsentChoice;

/// Emitted when a player selects a telemetry consent choice.
#[derive(Event, Debug, Clone, Copy, PartialEq, Eq)]
pub struct IndigaugeConsentSelectedEvent {
  pub choice: IndigaugeConsentChoice,
}
