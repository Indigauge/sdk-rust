use bytes::Bytes;
use indigauge_types::prelude::{ApiResponse, BatchEventPayload, FeedbackPayload, IndigaugeConfig, StartSessionPayload};
use reqwest::{Client, Method, Request, StatusCode, header::HeaderMap};
use serde::{Serialize, de::DeserializeOwned};
use serde_json::{Value, json};

/// Errors that can occur when building SDK HTTP requests.
#[derive(Debug)]
pub enum SdkBuildError {
  Serialize(String),
  Http(reqwest::Error),
}

impl std::fmt::Display for SdkBuildError {
  fn fmt(&self, f: &mut std::fmt::Formatter<'_>) -> std::fmt::Result {
    match self {
      SdkBuildError::Serialize(err) => write!(f, "serialize error: {}", err),
      SdkBuildError::Http(err) => write!(f, "http error: {}", err),
    }
  }
}

impl std::error::Error for SdkBuildError {}

impl From<reqwest::Error> for SdkBuildError {
  fn from(value: reqwest::Error) -> Self {
    Self::Http(value)
  }
}

impl From<serde_json::Error> for SdkBuildError {
  fn from(value: serde_json::Error) -> Self {
    Self::Serialize(value.to_string())
  }
}

/// Lightweight HTTP builder for Indigauge SDK operations.
pub struct SdkHttpClient<'a> {
  client: &'a Client,
  config: &'a IndigaugeConfig,
}

/// Response payload returned by SDK send helpers.
pub struct SdkResponse {
  pub body: Bytes,
  pub status: StatusCode,
  pub headers: HeaderMap,
}

#[derive(Debug, Clone, Copy, PartialEq, Eq)]
pub enum ResponseDisposition {
  Success,
  Failure,
}

/// Classifies an HTTP status code into success/failure for observer handling.
pub fn classify_status(status: StatusCode) -> ResponseDisposition {
  if status.is_success() {
    ResponseDisposition::Success
  } else {
    ResponseDisposition::Failure
  }
}

/// Decodes a UTF-8 response body.
pub fn decode_utf8_body(body: &[u8]) -> Result<&str, std::str::Utf8Error> {
  std::str::from_utf8(body)
}

/// Deserializes a JSON body into the requested type.
pub fn decode_json_body<T: DeserializeOwned>(body: &[u8]) -> Result<T, serde_json::Error> {
  serde_json::from_slice(body)
}

/// Deserializes a response body into the common API envelope.
pub fn decode_api_response<T: DeserializeOwned>(body: &[u8]) -> Result<ApiResponse<T>, serde_json::Error> {
  decode_json_body(body)
}

impl SdkResponse {
  /// Returns the raw response body bytes.
  pub fn body(&self) -> &Bytes {
    &self.body
  }

  /// Attempts to view the body as UTF-8 text.
  pub fn as_str(&self) -> Result<&str, std::str::Utf8Error> {
    decode_utf8_body(&self.body)
  }

  /// Deserializes the body as JSON into the requested type.
  pub fn deserialize_json<T: DeserializeOwned>(&self) -> Result<T, serde_json::Error> {
    decode_json_body(&self.body)
  }

  /// Returns the HTTP status code.
  pub fn status(&self) -> StatusCode {
    self.status
  }

  /// Returns the response headers.
  pub fn headers(&self) -> &HeaderMap {
    &self.headers
  }
}

impl<'a> SdkHttpClient<'a> {
  /// Creates a new request builder bound to a reqwest client and SDK config.
  pub fn new(client: &'a Client, config: &'a IndigaugeConfig) -> Self {
    Self { client, config }
  }

  /// Sends a built request and captures body, status, and headers.
  pub async fn send(&self, request: Request) -> Result<SdkResponse, reqwest::Error> {
    send_request(self.client, request).await
  }

  fn json_request<T: Serialize>(
    &self,
    method: Method,
    path: &str,
    api_key: &str,
    payload: &T,
  ) -> Result<Request, SdkBuildError> {
    let request = self
      .client
      .request(method, self.config.api_url(path))
      .timeout(self.config.request_timeout())
      .header("Content-Type", "application/json")
      .header("X-Indigauge-Key", api_key)
      .json(payload)
      .build()?;

    Ok(request)
  }

  /// Builds a request to start a session using the configured public key.
  pub fn start_session(&self, payload: &StartSessionPayload<'_>) -> Result<Request, SdkBuildError> {
    self.json_request(Method::POST, "sessions/start", self.config.public_key(), payload)
  }

  /// Builds a request to end an active session.
  pub fn end_session(&self, session_token: &str, reason: &str) -> Result<Request, SdkBuildError> {
    self.json_request(Method::POST, "sessions/end", session_token, &json!({ "reason": reason }))
  }

  /// Builds a heartbeat request for an active session.
  pub fn heartbeat(&self, session_token: &str) -> Result<Request, SdkBuildError> {
    self.json_request(Method::POST, "sessions/heartbeat", session_token, &json!({}))
  }

  /// Builds an event batch ingest request.
  pub fn event_batch(&self, session_token: &str, payload: &BatchEventPayload) -> Result<Request, SdkBuildError> {
    self.json_request(Method::POST, "events/batch", session_token, payload)
  }

  /// Builds a metadata update request.
  pub fn update_metadata<T: Serialize>(&self, session_token: &str, metadata: &T) -> Result<Request, SdkBuildError> {
    let value = serde_json::to_value(metadata)?;
    self.update_metadata_value(session_token, &value)
  }

  /// Builds a metadata update request from a pre-serialized JSON value.
  pub fn update_metadata_value(&self, session_token: &str, metadata: &Value) -> Result<Request, SdkBuildError> {
    self.json_request(Method::PATCH, "sessions", session_token, metadata)
  }

  /// Builds a feedback submission request.
  pub fn feedback(&self, session_token: &str, payload: &FeedbackPayload<'_>) -> Result<Request, SdkBuildError> {
    self.json_request(Method::POST, "feedback", session_token, payload)
  }

  /// Builds a screenshot upload request for an existing feedback record.
  pub fn feedback_screenshot(
    &self,
    session_token: &str,
    feedback_id: &str,
    png_bytes: Vec<u8>,
  ) -> Result<Request, SdkBuildError> {
    let path = format!("feedback/{}/screenshot", feedback_id);
    let request = self
      .client
      .post(self.config.api_url(&path))
      .timeout(self.config.request_timeout())
      .header("Content-Type", "image/png")
      .header("X-Indigauge-Key", session_token)
      .body(png_bytes)
      .build()?;

    Ok(request)
  }
}

/// Executes a request using the provided client and returns body + parts.
pub async fn send_request(client: &Client, request: Request) -> Result<SdkResponse, reqwest::Error> {
  let response = client.execute(request).await?;
  let status = response.status();
  let headers = response.headers().clone();
  let body = response.bytes().await?;
  Ok(SdkResponse { body, status, headers })
}
