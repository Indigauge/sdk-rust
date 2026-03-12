use bevy::prelude::*;
use indigauge_core::types::IndigaugeLogLevel;
use serde::Serialize;

use crate::{
  prelude::{EmptySessionMeta, StartSessionEvent},
  session::resources::{SessionApiKey, SessionMeta},
  utils::BevyIndigauge,
};

/// Ends the active session when exit events are observed.
pub fn handle_exit_event<E>(exit_events: MessageReader<E>, ig: BevyIndigauge, session_key: Option<Res<SessionApiKey>>)
where
  E: Message + std::fmt::Debug,
{
  if !exit_events.is_empty()
    && let Some(key) = session_key
  {
    end_session(ig, key);
  }
}

/// System to start a default session.
///
/// This is just a helper to start a default session. And will internally trigger the `StartSessionEvent`, like shown below.
/// If you need to initialize the session with a custom platform, please create your own system to initialize the session.
///
/// ```rust,ignore
/// commands.trigger(StartSessionEvent::default());
/// ```
///
/// # Example Usage:
///
/// ```rust,ignore
/// use bevy::prelude::*;
/// use bevy_mod_indigauge::prelude::*;
///
/// #[derive(Clone, Copy, PartialEq, Eq, Hash, Debug, Default, States)]
/// enum GameState {
///   #[default]
///   InitializeSession,
///   Playing
/// }
///
/// fn main() {
///   App::new()
///     .add_plugins(DefaultPlugins)
///     .insert_state(GameState::default())
///     .add_plugins(IndigaugePlugin::<EmptySessionMeta>::default().mode(IndigaugeMode::Dev))
///     .add_systems(OnEnter(GameState::InitializeSession), start_default_session)
///     .add_observer(switch_state_after_session_init(GameState::Playing))
///     .run();
/// }
/// ```
pub fn start_default_session(mut commands: Commands) {
  commands.trigger(StartSessionEvent::default());
}

/// System to end a session. Can be used to end a session manually.
///
/// # Example usage:
///
/// ```rust,ignore
/// use bevy::prelude::*;
/// use bevy_mod_indigauge::prelude::*;
///
/// #[derive(Clone, Copy, PartialEq, Eq, Hash, Debug, Default, States)]
/// enum GameState {
///   #[default]
///   InitializeSession,
///   Playing,
///   Ended
/// }
///
/// fn main() {
///   App::new()
///     .add_plugins(DefaultPlugins)
///     .insert_state(GameState::default())
///     .add_plugins(IndigaugePlugin::<EmptySessionMeta>::default().mode(IndigaugeMode::Dev))
///     .add_systems(OnEnter(GameState::InitializeSession), start_default_session)
///     .add_systems(OnEnter(GameState::Ended), end_session)
///     .add_observer(switch_state_after_session_init(GameState::Playing))
///     .run();
/// }
/// ```
pub fn end_session(mut ig: BevyIndigauge, session_key: Res<SessionApiKey>) {
  ig.flush_events(&session_key);

  match ig.sdk_client().end_session(&session_key, "ended") {
    Ok(request) => {
      ig.reqwest_client.send(request);
    },
    Err(error) => {
      if **ig.log_level <= IndigaugeLogLevel::Error {
        use bevy::log::error;
        error!(message = "Failed to build end session request", ?error);
      }
    },
  }
}

pub(crate) fn handle_updated_metadata<M>(mut session_meta: ResMut<SessionMeta<M>>)
where
  M: Resource + Serialize,
{
  session_meta.is_changed = true;
}

pub(crate) fn update_metadata<M>(
  mut session_meta: ResMut<SessionMeta<M>>,
  metadata: Option<Res<M>>,
  mut ig: BevyIndigauge,
  session_key: Option<Res<SessionApiKey>>,
) where
  M: Resource + Serialize,
{
  if session_meta.is_changed {
    session_meta.is_changed = false;

    if let Some(key) = session_key {
      if let Some(metadata_resource) = metadata {
        ig.update_metadata(&*metadata_resource, &key);
      } else {
        use bevy::log::warn;
        use std::any::type_name;

        let tn = type_name::<M>();
        if tn.ne(type_name::<EmptySessionMeta>()) && **ig.log_level <= IndigaugeLogLevel::Warn {
          warn!(message = "Metadata changed, but did not exist as a resource", type = tn);
        }
      }
    }
  }
}
