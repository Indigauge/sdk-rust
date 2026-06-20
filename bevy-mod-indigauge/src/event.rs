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

/// Plugin responsible for consuming queued events and flushing them periodically.
pub struct EventsPlugin {
  flush_interval: Duration,
}

impl EventsPlugin {
  /// Creates an event plugin with a custom flush interval.
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

#[cfg(test)]
mod tests {
  use bevy::ecs::schedule::ScheduleLabel;
  use bevy::ecs::system::RunSystemOnce;
  use bevy::render::RenderPlugin;
  use bevy::render::settings::{RenderCreation, WgpuSettings};

  use super::*;
  use crate::{
    plugin::IndigaugePlugin,
    prelude::{EmptySessionMeta, IndigaugeMode, StartSessionEvent, ig_debug, ig_error, ig_info, ig_warn},
  };

  fn get_master_app() -> App {
    let mut app = App::new();

    app.add_plugins(
      DefaultPlugins
        .set(WindowPlugin {
          primary_window: None,
          exit_condition: bevy::window::ExitCondition::DontExit,
          close_when_requested: false,
          ..default()
        })
        .set(RenderPlugin {
          render_creation: RenderCreation::Automatic(Box::new(WgpuSettings {
            backends: None,
            ..default()
          })),
          ..default()
        })
        .disable::<bevy::winit::WinitPlugin>(),
    );

    app
  }

  fn sub_app_log_system_one() {
    ig_info!("subapp.tick");
    ig_warn!("subapp.warn", { "source": "sub_app_log_system_one" });
  }

  fn sub_app_log_system_two() {
    ig_debug!("subapp.debug", { "source": "sub_app_log_system_two" });
    ig_error!("subapp.error");
  }

  #[derive(bevy::app::AppLabel, Clone, Copy, Debug, Eq, Hash, PartialEq)]
  struct LoggingSubApp;

  #[test]
  fn master_app_plugin_flushes_events_emitted_by_sub_app_systems() {
    let mut app = get_master_app();
    app.add_plugins(IndigaugePlugin::<EmptySessionMeta>::default().mode(IndigaugeMode::Dev));

    let mut sub_app = SubApp::new();
    sub_app.update_schedule = Some(Update.intern());
    sub_app.add_systems(Update, (sub_app_log_system_one, sub_app_log_system_two));
    app.insert_sub_app(LoggingSubApp, sub_app);

    app.world_mut().trigger(StartSessionEvent::default());

    app.update();

    assert!(app.world().contains_resource::<SessionApiKey>());
    assert!(app.world().contains_resource::<EventQueueReceiver>());
    assert_eq!(app.world().resource::<EventQueueReceiver>().len(), 4);
    assert_eq!(app.world().resource::<BufferedEvents>().events.len(), 0);
    assert!(
      !app
        .sub_app(LoggingSubApp)
        .world()
        .contains_resource::<EventQueueReceiver>()
    );

    app
      .world_mut()
      .run_system_once(handle_queued_events)
      .expect("queued events should be drained");
    assert_eq!(app.world().resource::<EventQueueReceiver>().len(), 0);
    assert_eq!(app.world().resource::<BufferedEvents>().events.len(), 4);

    app
      .world_mut()
      .run_system_once(flush_events)
      .expect("buffered events should flush");
    assert_eq!(app.world().resource::<BufferedEvents>().events.len(), 0);
  }
}
