pub(crate) use indigauge_core::hardware::{bucket_cores, bucket_ram_gb, coarsen_cpu_name};

#[cfg(feature = "panic_handler")]
pub(crate) use indigauge_core::panic::panic_handler_with_config as panic_handler;
