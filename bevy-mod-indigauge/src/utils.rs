use bevy::ecs::observer::On;
use bevy::ecs::system::{Res, ResMut, SystemParam};
use bevy::log::{error, info};
use indigauge_core::http::{ResponseDisposition, SdkHttpClient, classify_status};
use indigauge_types::prelude::BatchEventPayload;
use serde::Serialize;

use crate::config::*;
use crate::event::resources::BufferedEvents;
use crate::http_runtime::{BevyReqwest, ReqwestErrorEvent, ReqwestResponseEvent};

#[cfg(feature = "feedback")]
use indigauge_types::prelude::FeedbackPayload;

#[cfg(feature = "feedback")]
use bevy::ecs::{bundle::Bundle, system::IntoObserverSystem};

use indigauge_types::prelude::IndigaugeLogLevel;
use indigauge_types::prelude::IndigaugeMode;

fn log_response_outcome(
  log_level: &BevyIndigaugeLogLevel,
  status: reqwest::StatusCode,
  success_msg: &str,
  error_msg: &str,
) {
  match classify_status(status) {
    ResponseDisposition::Success => {
      if **log_level <= IndigaugeLogLevel::Info {
        info!(message = success_msg);
      }
    },
    ResponseDisposition::Failure => {
      if **log_level <= IndigaugeLogLevel::Error {
        error!(message = error_msg, ?status);
      }
    },
  }
}

fn log_transport_error(log_level: &BevyIndigaugeLogLevel, error_msg: &str, error: &reqwest::Error) {
  if **log_level <= IndigaugeLogLevel::Error {
    error!(message = error_msg, error = ?error);
  }
}

#[allow(unused)]
/// Returns `true_case` when `condition` is true, otherwise `false_case`.
pub fn select<T>(true_case: T, false_case: T, condition: bool) -> T {
  if condition { true_case } else { false_case }
}

/// System parameter bundling Indigauge resources and request client access.
#[derive(SystemParam)]
pub struct BevyIndigauge<'w, 's> {
  pub reqwest_client: BevyReqwest<'w, 's>,
  pub config: Res<'w, BevyIndigaugeConfig>,
  pub buffered_events: ResMut<'w, BufferedEvents>,
  pub log_level: Res<'w, BevyIndigaugeLogLevel>,
  pub mode: Res<'w, BevyIndigaugeMode>,
}

impl<'w, 's> BevyIndigauge<'w, 's> {
  pub(crate) fn sdk_client(&self) -> SdkHttpClient<'_> {
    SdkHttpClient::new(&self.reqwest_client, &self.config)
  }

  #[cfg(feature = "feedback")]
  pub(crate) fn send_feedback_screenshot(&mut self, api_key: &str, feedback_id: &str, image_data: Vec<u8>) {
    match **self.mode {
      IndigaugeMode::Live => match self.sdk_client().feedback_screenshot(api_key, feedback_id, image_data) {
        Ok(request) => {
          self
            .reqwest_client
            .send(request)
            .on_response(|trigger: On<ReqwestResponseEvent>, log_level: Res<BevyIndigaugeLogLevel>| {
              log_response_outcome(
                &log_level,
                trigger.status(),
                "Sent feedback screenshot",
                "Failed to send feedback screenshot",
              );
            })
            .on_error(|trigger: On<ReqwestErrorEvent>, log_level: Res<BevyIndigaugeLogLevel>| {
              log_transport_error(&log_level, "Failed to send feedback", &trigger.event().error);
            });
        },
        Err(error) => {
          if **self.log_level <= IndigaugeLogLevel::Error {
            error!(message = "Failed to build feedback screenshot request", ?error);
          }
        },
      },
      IndigaugeMode::Dev => {
        if **self.log_level <= IndigaugeLogLevel::Info {
          info!(message = "DEVMODE: Sent feedback screenshot");
        }
      },
      _ => {},
    }
  }

  #[cfg(feature = "feedback")]
  pub(crate) fn send_feedback<RB, RM, OR>(&mut self, api_key: &str, payload: &FeedbackPayload, on_response: OR)
  where
    RB: Bundle,
    OR: IntoObserverSystem<ReqwestResponseEvent, RB, RM>,
  {
    match **self.mode {
      IndigaugeMode::Live => match self.sdk_client().feedback(api_key, payload) {
        Ok(request) => {
          self.reqwest_client.send(request).on_response(on_response).on_error(
            |trigger: On<ReqwestErrorEvent>, log_level: Res<BevyIndigaugeLogLevel>| {
              log_transport_error(&log_level, "Failed to send feedback", &trigger.event().error);
            },
          );
        },
        Err(error) => {
          if **self.log_level <= IndigaugeLogLevel::Error {
            error!(message = "Failed to build feedback request", ?error);
          }
        },
      },
      IndigaugeMode::Dev => {
        if **self.log_level <= IndigaugeLogLevel::Info {
          info!(message = "DEVMODE: Sent feedback", feedback = ?payload);
        }
      },
      _ => {},
    }
  }

  pub(crate) fn flush_events(&mut self, api_key: &str) -> usize {
    let event_len = self.buffered_events.events.len();
    if event_len == 0 {
      return 0;
    }

    let events = BatchEventPayload {
      events: self
        .buffered_events
        .events
        .drain(..(event_len.min(self.config.batch_size())))
        .map(|event| event.into_inner())
        .collect::<Vec<_>>(),
    };

    match **self.mode {
      IndigaugeMode::Live => match self.sdk_client().event_batch(api_key, &events) {
        Ok(request) => {
          self
            .reqwest_client
            .send(request)
            .on_response(|trigger: On<ReqwestResponseEvent>, log_level: Res<BevyIndigaugeLogLevel>| {
              log_response_outcome(
                &log_level,
                trigger.event().status(),
                "Event batch sent successfully",
                "Failed to send event batch",
              );
            })
            .on_error(|trigger: On<ReqwestErrorEvent>, log_level: Res<BevyIndigaugeLogLevel>| {
              log_transport_error(&log_level, "Failed to send event batch", &trigger.event().error);
            });
        },
        Err(error) => {
          if **self.log_level <= IndigaugeLogLevel::Error {
            error!(message = "Failed to build event batch request", ?error);
          }
        },
      },
      IndigaugeMode::Dev => {
        if **self.log_level <= IndigaugeLogLevel::Info {
          info!(message = "DEVMODE: sending event batch", count = events.events.len());
        }
      },
      _ => {},
    }

    events.events.len()
  }

  pub(crate) fn send_heartbeat(&mut self, api_key: &str) {
    match **self.mode {
      IndigaugeMode::Live => match self.sdk_client().heartbeat(api_key) {
        Ok(request) => {
          self
            .reqwest_client
            .send(request)
            .on_response(|trigger: On<ReqwestResponseEvent>, log_level: Res<BevyIndigaugeLogLevel>| {
              log_response_outcome(
                &log_level,
                trigger.event().status(),
                "Heartbeat sent successfully",
                "Failed to update heartbeat",
              );
            })
            .on_error(|trigger: On<ReqwestErrorEvent>, log_level: Res<BevyIndigaugeLogLevel>| {
              log_transport_error(&log_level, "Failed to send session heartbeat", &trigger.event().error);
            });
        },
        Err(error) => {
          if **self.log_level <= IndigaugeLogLevel::Error {
            error!(message = "Failed to build heartbeat request", ?error);
          }
        },
      },
      IndigaugeMode::Dev => {
        if **self.log_level <= IndigaugeLogLevel::Info {
          info!("DEVMODE: heartbeat");
        }
      },
      _ => {},
    }
  }

  pub(crate) fn update_metadata<T>(&mut self, meta: &T, api_key: &str)
  where
    T: Serialize,
  {
    let metadata = match serde_json::to_value(meta) {
      Ok(json) => json,
      Err(error) => {
        if **self.log_level <= IndigaugeLogLevel::Error {
          error!(message = "Failed to serialize metadata", ?error);
        }
        return;
      },
    };

    match **self.mode {
      IndigaugeMode::Live => match self.sdk_client().update_metadata_value(api_key, &metadata) {
        Ok(request) => {
          self
            .reqwest_client
            .send(request)
            .on_response(|trigger: On<ReqwestResponseEvent>, log_level: Res<BevyIndigaugeLogLevel>| {
              log_response_outcome(
                &log_level,
                trigger.event().status(),
                "Metadata updated successfully",
                "Failed to update metadata",
              );
            })
            .on_error(|trigger: On<ReqwestErrorEvent>, log_level: Res<BevyIndigaugeLogLevel>| {
              log_transport_error(&log_level, "Failed to send session metadata update", &trigger.event().error);
            });
        },
        Err(error) => {
          if **self.log_level <= IndigaugeLogLevel::Error {
            error!(message = "Failed to build metadata request", ?error);
          }
        },
      },
      IndigaugeMode::Dev => {
        if **self.log_level <= IndigaugeLogLevel::Info {
          info!(message = "DEVMODE: update metadata", ?metadata);
        }
      },
      _ => {},
    }
  }

  #[cfg(not(target_family = "wasm"))]
  pub(crate) fn get_or_init_player_id(&self) -> String {
    use std::fs;
    use uuid::Uuid;
    let game_folder_path = dirs::preference_dir().map(|dir| dir.join(self.config.game_name()));

    if let Some(game_folder_path) = game_folder_path {
      let player_id_file_path = game_folder_path.join("player_id.txt");

      if let Ok(player_id) = fs::read_to_string(&player_id_file_path) {
        player_id
      } else {
        let new_player_id = Uuid::new_v4().to_string();
        let _ = fs::create_dir_all(&game_folder_path);
        let _ = fs::write(&player_id_file_path, &new_player_id);
        new_player_id
      }
    } else {
      Uuid::new_v4().to_string()
    }
  }
}
