use bevy::ecs::observer::On;
use bevy::ecs::system::{Res, ResMut, SystemParam};
use bevy::log::{error, info};
use indigauge_core::http::{
  ResponseDisposition, SdkHttpClient, get_or_init_player_id, response_disposition_for_level, should_log_transport_error,
};
use indigauge_core::types::BatchEventPayload;
use serde::Serialize;

use crate::config::*;
use crate::event::resources::BufferedEvents;
use crate::http_runtime::{BevyReqwest, ReqwestErrorEvent, ReqwestResponseEvent};

#[cfg(feature = "feedback")]
use indigauge_core::types::FeedbackPayload;

#[cfg(feature = "feedback")]
use bevy::ecs::{bundle::Bundle, system::IntoObserverSystem};

use indigauge_core::types::{IndigaugeLogLevel, IndigaugeMode};

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
              match response_disposition_for_level(&log_level, trigger.status()) {
                Some(ResponseDisposition::Success) => info!(message = "Sent feedback screenshot"),
                Some(ResponseDisposition::Failure) => error!(message = "Failed to send feedback screenshot"),
                None => {},
              }
            })
            .on_error(|trigger: On<ReqwestErrorEvent>, log_level: Res<BevyIndigaugeLogLevel>| {
              if should_log_transport_error(&log_level) {
                error!(message = "Failed to send feedback", error = ?trigger.event().error);
              }
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
              if should_log_transport_error(&log_level) {
                error!(message = "Failed to send feedback", error = ?trigger.event().error);
              }
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
              match response_disposition_for_level(&log_level, trigger.event().status()) {
                Some(ResponseDisposition::Success) => info!(message = "Event batch sent successfully"),
                Some(ResponseDisposition::Failure) => {
                  let status = trigger.event().status();
                  error!(message = "Failed to send event batch", ?status);
                },
                None => {},
              }
            })
            .on_error(|trigger: On<ReqwestErrorEvent>, log_level: Res<BevyIndigaugeLogLevel>| {
              if should_log_transport_error(&log_level) {
                error!(message = "Failed to send event batch", error = ?trigger.event().error);
              }
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
              match response_disposition_for_level(&log_level, trigger.event().status()) {
                Some(ResponseDisposition::Success) => info!(message = "Heartbeat sent successfully"),
                Some(ResponseDisposition::Failure) => {
                  let status = trigger.event().status();
                  error!(message = "Failed to update heartbeat", ?status);
                },
                None => {},
              }
            })
            .on_error(|trigger: On<ReqwestErrorEvent>, log_level: Res<BevyIndigaugeLogLevel>| {
              if should_log_transport_error(&log_level) {
                error!(message = "Failed to send session heartbeat", error = ?trigger.event().error);
              }
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
              match response_disposition_for_level(&log_level, trigger.event().status()) {
                Some(ResponseDisposition::Success) => info!(message = "Metadata updated successfully"),
                Some(ResponseDisposition::Failure) => {
                  let status = trigger.event().status();
                  error!(message = "Failed to update metadata", ?status);
                },
                None => {},
              }
            })
            .on_error(|trigger: On<ReqwestErrorEvent>, log_level: Res<BevyIndigaugeLogLevel>| {
              if should_log_transport_error(&log_level) {
                error!(message = "Failed to send session metadata update", error = ?trigger.event().error);
              }
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
    get_or_init_player_id(self.config.game_name())
  }
}
