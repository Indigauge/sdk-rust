#![doc = include_str!("../README.md")]

pub(crate) mod utils;

pub(crate) mod config;
pub(crate) mod event;
pub(crate) mod http_runtime;

#[cfg(feature = "feedback")]
pub(crate) mod feedback;

pub mod plugin;
pub(crate) mod session;

#[cfg(feature = "tracing")]
pub mod tracing;

pub mod prelude {
  pub use indigauge_types::prelude::{IndigaugeLogLevel, IndigaugeMode};

  pub use crate::config::{BevyIndigaugeLogLevel, BevyIndigaugeMode};
  pub use crate::event::utils::enqueue;
  pub use indigauge_core::event::validate_event_type_compile_time;
  pub use indigauge_core::{enqueue_ig_event, ig_debug, ig_error, ig_event, ig_info, ig_trace, ig_warn};

  #[cfg(feature = "feedback")]
  pub use crate::feedback::observers::{switch_state_on_feedback_despawn, switch_state_on_feedback_spawn};
  #[cfg(feature = "feedback")]
  pub use crate::feedback::{
    resources::{FeedbackKeyCodeToggle, FeedbackPanelProps, FeedbackPanelStyles},
    types::{FeedbackCategory, FeedbackSpawnPosition},
  };
  pub use crate::plugin::IndigaugePlugin;
  pub use crate::session::observers::switch_state_after_session_init;
  pub use crate::session::systems::{end_session, start_default_session};
  pub use crate::session::{
    events::{IndigaugeInitDoneEvent, StartSessionEvent},
    resources::EmptySessionMeta,
  };
}
