use serde::Deserialize;
use std::ops::Deref;

#[derive(Debug, Deserialize)]
#[serde(untagged)]
pub enum ApiResponse<T, E = ErrorBody> {
  Ok(T),
  Err(E),
}

#[derive(Debug, Deserialize)]
pub struct ErrorBody {
  pub code: String,
  pub message: String,
}

#[allow(unused)]
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
