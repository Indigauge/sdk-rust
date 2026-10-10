#![doc = include_str!("../README.md")]

pub mod event;
pub mod hardware;
pub mod http;
pub mod runtime;
pub mod state;
pub mod types {
  pub use indigauge_types::prelude::*;
}
pub mod utils;

#[cfg(feature = "panic_handler")]
pub mod panic;

#[cfg(feature = "tracing")]
pub mod tracing;

pub mod prelude {
  pub use crate::event::{QueuedEvent, validate_event_type, validate_event_type_compile_time};
  pub use crate::hardware::{bucket_cores, bucket_ram_gb, coarsen_cpu_name};
  pub use crate::http::{
    ResponseDisposition, SdkBuildError, SdkHttpClient, SdkResponse, classify_status, decode_api_response,
    decode_json_body, decode_utf8_body, response_disposition_for_level, send_request, should_log_transport_error,
  };
  #[cfg(not(target_family = "wasm"))]
  pub use crate::http::{SdkBlockingHttpClient, send_request_blocking};
  #[cfg(not(target_family = "wasm"))]
  pub use crate::runtime::IndigaugeBlockingRuntimeClient;
  pub use crate::runtime::IndigaugeRuntimeClient;
  pub use crate::state::{clear_pending_event_count, drain_pending_events, track_pending_event};
  pub use crate::types::*;
  pub use crate::utils::select;
  pub use crate::{
    enqueue_ig_event, ig_debug, ig_debug_once, ig_error, ig_error_once, ig_event, ig_event_once, ig_info, ig_info_once,
    ig_trace, ig_trace_once, ig_warn, ig_warn_once,
  };

  #[cfg(feature = "panic_handler")]
  pub use crate::panic::panic_handler_with_config;

  #[cfg(feature = "tracing")]
  pub use crate::tracing::{IndigaugeLayer, IndigaugeSink};
}
