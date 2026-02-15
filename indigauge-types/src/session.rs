use serde::{Deserialize, Serialize};

/// Payload sent when creating a new analytics session.
#[derive(Serialize, Debug)]
#[serde(rename_all = "camelCase")]
pub struct StartSessionPayload<'a> {
  pub client_version: &'a str,
  pub sdk_version: &'a str,
  pub player_id: Option<&'a String>,
  pub platform: Option<&'a String>,
  pub os: Option<&'a str>,
  pub cpu_family: Option<&'a String>,
  pub cores: Option<&'a str>,
  pub memory: Option<&'a str>,
  pub gpu: Option<&'a String>,
}

/// Response returned by the session start endpoint.
#[derive(Deserialize, Debug)]
#[serde(rename_all = "camelCase")]
pub struct StartSessionResponse {
  pub session_token: String,
}

impl StartSessionResponse {
  /// Returns a deterministic development-mode session response.
  pub fn dev() -> Self {
    Self {
      session_token: "dev".to_string(),
    }
  }
}
