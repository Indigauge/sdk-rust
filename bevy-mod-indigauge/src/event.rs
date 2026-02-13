use std::time::Duration;

use bevy::{prelude::*, time::common_conditions::on_timer};

use crate::{
  event::{
    resources::{BufferedEvents, EventQueueReceiver},
    systems::*,
  },
  session::resources::SessionApiKey,
};

pub(crate) mod resources;
mod systems;
pub(crate) mod utils;

pub struct EventsPlugin {
  flush_interval: Duration,
}

impl EventsPlugin {
  pub fn new(flush_interval: Duration) -> Self {
    Self { flush_interval }
  }
}

impl Plugin for EventsPlugin {
  fn build(&self, app: &mut App) {
    app.add_systems(
      Update,
      (
        handle_queued_events.run_if(resource_exists::<EventQueueReceiver>),
        maybe_flush_events.run_if(resource_changed::<BufferedEvents>),
        flush_events.run_if(on_timer(self.flush_interval)),
      )
        .run_if(resource_exists::<SessionApiKey>),
    );
  }
}
