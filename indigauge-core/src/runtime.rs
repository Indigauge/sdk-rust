use indigauge_types::prelude::{
  BatchEventPayload, EventPayload, FeedbackPayload, IndigaugeConfig, StartSessionPayload,
};
use reqwest::{Client, Request};

#[cfg(not(target_family = "wasm"))]
use reqwest::blocking::{Client as BlockingClient, Request as BlockingRequest};

use crate::http::{SdkBuildError, SdkHttpClient, SdkResponse};

#[cfg(not(target_family = "wasm"))]
use crate::http::SdkBlockingHttpClient;

/// Framework-agnostic async runtime client that owns config and transport.
/// Use this in engines that already run an async executor.
pub struct IndigaugeRuntimeClient {
  config: IndigaugeConfig,
  client: Client,
}

impl IndigaugeRuntimeClient {
  /// Creates a runtime client with a default reqwest async client.
  pub fn new(config: IndigaugeConfig) -> Self {
    Self {
      config,
      client: Client::new(),
    }
  }

  /// Creates a runtime client with a caller-supplied reqwest async client.
  pub fn with_client(config: IndigaugeConfig, client: Client) -> Self {
    Self { config, client }
  }

  /// Returns the SDK config.
  pub fn config(&self) -> &IndigaugeConfig {
    &self.config
  }

  /// Returns the underlying reqwest async client.
  pub fn client(&self) -> &Client {
    &self.client
  }

  /// Returns a low-level HTTP helper bound to this runtime client.
  pub fn http(&self) -> SdkHttpClient<'_> {
    SdkHttpClient::new(&self.client, &self.config)
  }

  /// Sends an already built request.
  pub async fn send(&self, request: Request) -> Result<SdkResponse, reqwest::Error> {
    self.http().send(request).await
  }

  /// Builds a start session request.
  pub fn start_session(&self, payload: &StartSessionPayload<'_>) -> Result<Request, SdkBuildError> {
    self.http().start_session(payload)
  }

  /// Builds an end session request.
  pub fn end_session(&self, session_token: &str, reason: &str) -> Result<Request, SdkBuildError> {
    self.http().end_session(session_token, reason)
  }

  /// Builds a heartbeat request.
  pub fn heartbeat(&self, session_token: &str) -> Result<Request, SdkBuildError> {
    self.http().heartbeat(session_token)
  }

  /// Builds an event batch request.
  pub fn event_batch(&self, session_token: &str, payload: &BatchEventPayload) -> Result<Request, SdkBuildError> {
    self.http().event_batch(session_token, payload)
  }

  /// Builds a single event request.
  pub fn event(&self, session_token: &str, payload: &EventPayload) -> Result<Request, SdkBuildError> {
    self.http().event(session_token, payload)
  }

  /// Builds a metadata update request.
  pub fn update_metadata<T: serde::Serialize>(
    &self,
    session_token: &str,
    metadata: &T,
  ) -> Result<Request, SdkBuildError> {
    self.http().update_metadata(session_token, metadata)
  }

  /// Builds a metadata update request from a JSON value.
  pub fn update_metadata_value(
    &self,
    session_token: &str,
    metadata: &serde_json::Value,
  ) -> Result<Request, SdkBuildError> {
    self.http().update_metadata_value(session_token, metadata)
  }

  /// Builds a feedback request.
  pub fn feedback(&self, session_token: &str, payload: &FeedbackPayload<'_>) -> Result<Request, SdkBuildError> {
    self.http().feedback(session_token, payload)
  }

  /// Builds a feedback screenshot request.
  pub fn feedback_screenshot(
    &self,
    session_token: &str,
    feedback_id: &str,
    png_bytes: Vec<u8>,
  ) -> Result<Request, SdkBuildError> {
    self.http().feedback_screenshot(session_token, feedback_id, png_bytes)
  }
}

/// Framework-agnostic blocking runtime client for native game loops and crash paths.
#[cfg(not(target_family = "wasm"))]
pub struct IndigaugeBlockingRuntimeClient {
  config: IndigaugeConfig,
  client: BlockingClient,
}

#[cfg(not(target_family = "wasm"))]
impl IndigaugeBlockingRuntimeClient {
  /// Creates a blocking runtime client with a default reqwest blocking client.
  pub fn new(config: IndigaugeConfig) -> Self {
    Self {
      config,
      client: BlockingClient::new(),
    }
  }

  /// Creates a blocking runtime client with a caller-supplied blocking client.
  pub fn with_client(config: IndigaugeConfig, client: BlockingClient) -> Self {
    Self { config, client }
  }

  /// Returns the SDK config.
  pub fn config(&self) -> &IndigaugeConfig {
    &self.config
  }

  /// Returns the underlying reqwest blocking client.
  pub fn client(&self) -> &BlockingClient {
    &self.client
  }

  /// Returns a low-level blocking HTTP helper bound to this runtime client.
  pub fn http(&self) -> SdkBlockingHttpClient<'_> {
    SdkBlockingHttpClient::new(&self.client, &self.config)
  }

  /// Sends an already built blocking request.
  pub fn send(&self, request: BlockingRequest) -> Result<SdkResponse, reqwest::Error> {
    self.http().send(request)
  }

  /// Builds a start session request.
  pub fn start_session(&self, payload: &StartSessionPayload<'_>) -> Result<BlockingRequest, SdkBuildError> {
    self.http().start_session(payload)
  }

  /// Builds an end session request.
  pub fn end_session(&self, session_token: &str, reason: &str) -> Result<BlockingRequest, SdkBuildError> {
    self.http().end_session(session_token, reason)
  }

  /// Builds a heartbeat request.
  pub fn heartbeat(&self, session_token: &str) -> Result<BlockingRequest, SdkBuildError> {
    self.http().heartbeat(session_token)
  }

  /// Builds an event batch request.
  pub fn event_batch(
    &self,
    session_token: &str,
    payload: &BatchEventPayload,
  ) -> Result<BlockingRequest, SdkBuildError> {
    self.http().event_batch(session_token, payload)
  }

  /// Builds a single event request.
  pub fn event(&self, session_token: &str, payload: &EventPayload) -> Result<BlockingRequest, SdkBuildError> {
    self.http().event(session_token, payload)
  }

  /// Builds a metadata update request.
  pub fn update_metadata<T: serde::Serialize>(
    &self,
    session_token: &str,
    metadata: &T,
  ) -> Result<BlockingRequest, SdkBuildError> {
    self.http().update_metadata(session_token, metadata)
  }

  /// Builds a metadata update request from a JSON value.
  pub fn update_metadata_value(
    &self,
    session_token: &str,
    metadata: &serde_json::Value,
  ) -> Result<BlockingRequest, SdkBuildError> {
    self.http().update_metadata_value(session_token, metadata)
  }

  /// Builds a feedback request.
  pub fn feedback(&self, session_token: &str, payload: &FeedbackPayload<'_>) -> Result<BlockingRequest, SdkBuildError> {
    self.http().feedback(session_token, payload)
  }

  /// Builds a feedback screenshot request.
  pub fn feedback_screenshot(
    &self,
    session_token: &str,
    feedback_id: &str,
    png_bytes: Vec<u8>,
  ) -> Result<BlockingRequest, SdkBuildError> {
    self.http().feedback_screenshot(session_token, feedback_id, png_bytes)
  }
}
