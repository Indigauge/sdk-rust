#[cfg(all(feature = "panic_handler", not(target_family = "wasm")))]
pub(crate) use indigauge_core::panic::panic_handler_with_config as panic_handler;
