use bevy::prelude::*;
use bevy::render::RenderPlugin;
use bevy::render::settings::{RenderCreation, WgpuSettings};
use bevy_mod_indigauge::prelude::*;

#[derive(Resource, serde::Serialize, Clone, Copy, Default)]
struct MyMeta;

fn get_app() -> App {
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
        // Disable GPU backend
        render_creation: RenderCreation::Automatic(WgpuSettings {
          backends: None,
          ..default()
        }),
        ..default()
      })
      .disable::<bevy::winit::WinitPlugin>(),
  );
  app
}

#[test]
fn test_plugin_init() {
  let mut app = get_app();
  app.add_plugins(IndigaugePlugin::<MyMeta>::default());

  // Run one update to trigger startup systems
  app.update();

  assert!(app.world().contains_resource::<BevyIndigaugeMode>());
}

#[test]
fn test_disabled_mode() {
  let mut app = get_app();
  app.add_plugins(IndigaugePlugin::<MyMeta>::default().mode(IndigaugeMode::Disabled));

  app.update();

  let mode = app.world().resource::<BevyIndigaugeMode>();
  assert_eq!(**mode, IndigaugeMode::Disabled);
}
