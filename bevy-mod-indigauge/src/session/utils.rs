pub(crate) use indigauge_core::hardware::{bucket_cores, bucket_ram_gb, coarsen_cpu_name};

#[cfg(all(feature = "panic_handler", not(target_family = "wasm")))]
pub(crate) use indigauge_core::panic::panic_handler;
