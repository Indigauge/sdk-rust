use bevy::prelude::*;
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

#[test]
fn multiple_apps_emit_ig_events_with_distinct_systems() {
  DISPATCHER_INIT.call_once(|| {
    set_event_dispatcher(capture_event);
  });

  let events = EVENTS.get_or_init(|| Mutex::new(Vec::new()));
  events.lock().expect("event collector mutex poisoned").clear();

  let mut app_one = App::new();
  app_one.add_plugins(MinimalPlugins);
  app_one.add_systems(Update, app_one_system);

  let mut app_two = App::new();
  app_two.add_plugins(MinimalPlugins);
  app_two.add_systems(Update, app_two_system);

  // Interleave updates to represent both apps running side-by-side in one process.
  for _ in 0..3 {
    app_one.update();
    app_two.update();
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
