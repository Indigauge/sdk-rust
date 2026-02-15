use bevy::ecs::observer::On;
use bevy::ecs::system::{Res, ResMut, SystemParam};
use bevy::log::{error, info};
use bevy_mod_reqwest::reqwest::{Error as ReqwestError, Request};
use bevy_mod_reqwest::{BevyReqwest, ReqwestErrorEvent, ReqwestResponseEvent};
use indigauge_types::prelude::BatchEventPayload;
use serde::Serialize;
use serde_json::json;

use crate::config::*;
use crate::event::resources::BufferedEvents;

#[cfg(feature = "feedback")]
use indigauge_types::prelude::FeedbackPayload;

#[cfg(feature = "feedback")]
use bevy::ecs::{bundle::Bundle, system::IntoObserverSystem};

use indigauge_types::prelude::IndigaugeLogLevel;
use indigauge_types::prelude::IndigaugeMode;

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
  pub(crate) fn build_post_request<S>(&self, url: &str, ig_key: &str, payload: &S) -> Result<Request, ReqwestError>
  where
    S: Serialize,
  {
    self
      .reqwest_client
      .post(self.config.api_url(url))
      .timeout(self.config.request_timeout())
      .header("Content-Type", "application/json")
      .header("X-Indigauge-Key", ig_key)
      .json(payload)
      .build()
  }

  pub(crate) fn build_patch_request<S>(&self, url: &str, ig_key: &str, payload: &S) -> Result<Request, ReqwestError>
  where
    S: Serialize,
  {
    self
      .reqwest_client
      .patch(self.config.api_url(url))
      .timeout(self.config.request_timeout())
      .header("Content-Type", "application/json")
      .header("X-Indigauge-Key", ig_key)
      .json(payload)
      .build()
  }

  #[cfg(feature = "feedback")]
  pub(crate) fn send_feedback_screenshot(&mut self, api_key: &str, feedback_id: &str, image_data: Vec<u8>) {
    match **self.mode {
      IndigaugeMode::Live => {
        let screenshot_path = format!("feedback/{}/screenshot", feedback_id);

        let request = self
          .reqwest_client
          .post(self.config.api_url(&screenshot_path))
          .timeout(self.config.request_timeout())
          .header("Content-Type", "image/png")
          .header("X-Indigauge-Key", api_key)
          .body(image_data)
          .build();

        if let Ok(request) = request {
          self
            .reqwest_client
            .send(request)
            .on_response(|trigger: On<ReqwestResponseEvent>, log_level: Res<BevyIndigaugeLogLevel>| {
              if trigger.status().is_success() {
                if **log_level <= IndigaugeLogLevel::Info {
                  info!(message = "Sent feedback screenshot");
                }
              } else if **log_level <= IndigaugeLogLevel::Error {
                error!(message = "Failed to send feedback screenshot");
              }
            })
            .on_error(|trigger: On<ReqwestErrorEvent>, log_level: Res<BevyIndigaugeLogLevel>| {
              if **log_level <= IndigaugeLogLevel::Error {
                error!(message = "Failed to send feedback", error = ?trigger.event().error);
              }
            });
        }
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
      IndigaugeMode::Live => {
        if let Ok(request) = self.build_post_request("feedback", api_key, payload) {
          self.reqwest_client.send(request).on_response(on_response).on_error(
            |trigger: On<ReqwestErrorEvent>, log_level: Res<BevyIndigaugeLogLevel>| {
              if **log_level <= IndigaugeLogLevel::Error {
                error!(message = "Failed to send feedback", error = ?trigger.event().error);
              }
            },
          );
        }
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
      IndigaugeMode::Live => {
        if let Ok(request) = self.build_post_request("events/batch", api_key, &events) {
          self
            .reqwest_client
            .send(request)
            .on_response(|trigger: On<ReqwestResponseEvent>, log_level: Res<BevyIndigaugeLogLevel>| {
              let status = trigger.event().status();
              if status.is_success() {
                if **log_level <= IndigaugeLogLevel::Info {
                  info!(message = "Event batch sent successfully");
                }
              } else if **log_level <= IndigaugeLogLevel::Error {
                error!(message = "Failed to send event batch", ?status);
              }
            })
            .on_error(|trigger: On<ReqwestErrorEvent>, log_level: Res<BevyIndigaugeLogLevel>| {
              if **log_level <= IndigaugeLogLevel::Error {
                error!(message = "Failed to send event batch", error = ?trigger.event().error);
              }
            });
        }
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
      IndigaugeMode::Live => {
        if let Ok(request) = self.build_post_request("sessions/heartbeat", api_key, &json!({})) {
          self
            .reqwest_client
            .send(request)
            .on_response(|trigger: On<ReqwestResponseEvent>, log_level: Res<BevyIndigaugeLogLevel>| {
              let status = trigger.event().status();
              if status.is_success() {
                if **log_level <= IndigaugeLogLevel::Info {
                  info!(message = "Heartbeat sent successfully");
                }
              } else if **log_level <= IndigaugeLogLevel::Error {
                error!(message = "Failed to update heartbeat", ?status);
              }
            })
            .on_error(|trigger: On<ReqwestErrorEvent>, log_level: Res<BevyIndigaugeLogLevel>| {
              if **log_level <= IndigaugeLogLevel::Error {
                error!(message = "Failed to send session heartbeat", error = ?trigger.event().error);
              }
            });
        }
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
      IndigaugeMode::Live => {
        if let Ok(request) = self.build_patch_request("sessions", api_key, &metadata) {
          self
            .reqwest_client
            .send(request)
            .on_response(|trigger: On<ReqwestResponseEvent>, log_level: Res<BevyIndigaugeLogLevel>| {
              let status = trigger.event().status();
              if status.is_success() {
                if **log_level <= IndigaugeLogLevel::Info {
                  info!(message = "Metadata updated successfully");
                }
              } else if **log_level <= IndigaugeLogLevel::Error {
                error!(message = "Failed to update metadata", ?status);
              }
            })
            .on_error(|trigger: On<ReqwestErrorEvent>, log_level: Res<BevyIndigaugeLogLevel>| {
              if **log_level <= IndigaugeLogLevel::Error {
                error!(message = "Failed to send session metadata update", error = ?trigger.event().error);
              }
            });
        }
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
