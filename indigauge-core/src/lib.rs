pub mod event;
pub mod hardware;

#[cfg(feature = "panic_handler")]
pub mod panic;

#[cfg(feature = "tracing")]
pub mod tracing;

pub mod prelude {
  pub use crate::event::{QueuedEvent, validate_event_type, validate_event_type_compile_time};
  pub use crate::hardware::{bucket_cores, bucket_ram_gb, coarsen_cpu_name};

  #[cfg(feature = "panic_handler")]
  pub use crate::panic::panic_handler;

  #[cfg(feature = "tracing")]
  pub use crate::tracing::{IndigaugeLayer, IndigaugeSink};
}
