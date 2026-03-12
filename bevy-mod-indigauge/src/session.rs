use std::marker::PhantomData;
use std::time::Duration;

use bevy::{prelude::*, time::common_conditions::on_timer, window::WindowCloseRequested};
use serde::Serialize;

use crate::{
  session::observers::observe_start_session_event,
  session::resources::{SessionApiKey, SessionMeta},
  session::systems::{handle_exit_event, handle_updated_metadata, update_metadata},
};

pub mod events;
pub(crate) mod observers;
pub mod resources;
pub(crate) mod systems;
pub mod utils;

/// Plugin that manages session lifecycle, metadata updates, and exit handling.
pub struct SessionPlugin<M: Resource + Serialize> {
  m: PhantomData<M>,
  flush_interval: Duration,
}

impl<M> SessionPlugin<M>
where
  M: Resource + Serialize,
{
  /// Creates a new session plugin with the provided flush interval.
  pub fn new(flush_interval: Duration) -> Self {
    Self {
      m: Default::default(),
      flush_interval,
    }
  }
}

impl<M> Plugin for SessionPlugin<M>
where
  M: Resource + Serialize,
{
  fn build(&self, app: &mut App) {
    app
      .insert_resource(SessionMeta::<M>::default())
      .add_observer(observe_start_session_event)
      .add_systems(
        Update,
        (
          handle_updated_metadata::<M>.run_if(resource_exists_and_changed::<M>),
          update_metadata::<M>.run_if(on_timer(self.flush_interval)),
        )
          .run_if(resource_exists::<SessionApiKey>),
      )
      .add_systems(
        PostUpdate,
        (handle_exit_event::<AppExit>, handle_exit_event::<WindowCloseRequested>)
          .run_if(resource_exists::<SessionApiKey>),
      );
  }
}
