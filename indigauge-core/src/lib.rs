#![doc = include_str!("../README.md")]

pub mod event;
pub mod hardware;
pub mod http;
pub mod state;
pub mod types {
  pub use indigauge_types::prelude::*;
}

#[cfg(feature = "panic_handler")]
pub mod panic;

#[cfg(feature = "tracing")]
pub mod tracing;

pub mod prelude {
  pub use crate::event::{QueuedEvent, validate_event_type, validate_event_type_compile_time};
  pub use crate::hardware::{bucket_cores, bucket_ram_gb, coarsen_cpu_name};
  pub use crate::http::{
    ResponseDisposition, SdkBuildError, SdkHttpClient, SdkResponse, classify_status, decode_api_response,
    decode_json_body, decode_utf8_body, response_disposition_for_level, select, send_request,
    should_log_transport_error,
  };
  pub use crate::types::*;
  pub use crate::{enqueue_ig_event, ig_debug, ig_error, ig_event, ig_info, ig_trace, ig_warn};

  #[cfg(feature = "panic_handler")]
  pub use crate::panic::panic_handler;

  #[cfg(feature = "tracing")]
  pub use crate::tracing::{IndigaugeLayer, IndigaugeSink};
}
