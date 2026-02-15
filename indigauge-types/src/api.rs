use serde::Deserialize;
use std::ops::Deref;

/// Generic API envelope used by Indigauge endpoints.
#[derive(Debug, Deserialize)]
#[serde(untagged)]
pub enum ApiResponse<T, E = ErrorBody> {
  Ok(T),
  Err(E),
}

/// Error response body returned for failed API calls.
#[derive(Debug, Deserialize)]
pub struct ErrorBody {
  pub code: String,
  pub message: String,
}

#[allow(unused)]
/// Response containing only a generated identifier.
#[derive(Debug, Deserialize)]
pub struct IdResponse {
  pub id: String,
}

impl Deref for IdResponse {
  type Target = String;

  fn deref(&self) -> &Self::Target {
    &self.id
  }
}
