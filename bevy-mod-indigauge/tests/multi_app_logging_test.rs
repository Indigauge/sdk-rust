use bevy::ecs::schedule::ScheduleLabel;
use bevy::prelude::*;
use bevy::render::RenderPlugin;
use bevy::render::settings::{RenderCreation, WgpuSettings};
use bevy_mod_indigauge::prelude::*;
use indigauge_core::event::set_event_dispatcher;
use serde_json::Value;
use std::sync::{Mutex, Once, OnceLock};

#[derive(Clone, Debug)]
struct CapturedEvent {
  level: &'static str,
  event_type: String,
}

static EVENTS: OnceLock<Mutex<Vec<CapturedEvent>>> = OnceLock::new();
static DISPATCHER_INIT: Once = Once::new();

fn capture_event(
  level: &'static str,
  event_type: &str,
  _metadata: Option<Value>,
  _file: &'static str,
  _line: u32,
  _module: &'static str,
) -> bool {
  let events = EVENTS.get_or_init(|| Mutex::new(Vec::new()));
  events
    .lock()
    .expect("event collector mutex poisoned")
    .push(CapturedEvent {
      level,
      event_type: event_type.to_string(),
    });
  true
}

fn app_one_system() {
  ig_info!("appone.tick");
  ig_warn!("appone.warn", { "source": "app_one_system" });
}

fn app_two_system() {
  ig_debug!("apptwo.tick", { "source": "app_two_system" });
  ig_error!("apptwo.error");
}

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

#[derive(bevy::app::AppLabel, Clone, Copy, Debug, Eq, Hash, PartialEq)]
struct LoggingSubApp;

#[test]
fn master_app_plugin_does_not_block_sub_app_ig_events() {
  DISPATCHER_INIT.call_once(|| {
    set_event_dispatcher(capture_event);
  });

  let events = EVENTS.get_or_init(|| Mutex::new(Vec::new()));
  events.lock().expect("event collector mutex poisoned").clear();

  let mut master_app = get_master_app();
  master_app.add_plugins(IndigaugePlugin::<EmptySessionMeta>::default().mode(IndigaugeMode::Disabled));
  master_app.add_systems(Update, app_one_system);

  let mut sub_app = SubApp::new();
  sub_app.update_schedule = Some(Update.intern());
  sub_app.add_systems(Update, app_two_system);
  master_app.insert_sub_app(LoggingSubApp, sub_app);

  for _ in 0..3 {
    master_app.update();
  }

  let captured = events.lock().expect("event collector mutex poisoned");

  assert!(
    captured
      .iter()
      .any(|event| event.event_type == "appone.tick" && event.level == "info")
  );
  assert!(
    captured
      .iter()
      .any(|event| event.event_type == "appone.warn" && event.level == "warn")
  );
  assert!(
    captured
      .iter()
      .any(|event| event.event_type == "apptwo.tick" && event.level == "debug")
  );
  assert!(
    captured
      .iter()
      .any(|event| event.event_type == "apptwo.error" && event.level == "error")
  );
}
