#[cfg(not(target_family = "wasm"))]
use std::path::PathBuf;

#[cfg(not(target_family = "wasm"))]
const CONSENT_FILE_NAME: &str = "telemetry_consent.txt";

#[cfg(not(target_family = "wasm"))]
pub(crate) fn consent_file_path(game_name: &str) -> Option<PathBuf> {
  dirs::preference_dir().map(|dir| dir.join(game_name).join(CONSENT_FILE_NAME))
}
