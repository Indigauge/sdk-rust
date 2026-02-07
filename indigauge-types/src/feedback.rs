use serde::Serialize;

#[derive(Serialize, Clone, Debug)]
#[serde(rename_all = "camelCase")]
pub struct FeedbackPayload<'a> {
  pub message: &'a str,
  /// Defaults to elapsed time since session start
  pub elapsed_ms: u128,
  pub question: Option<&'a String>,
  pub category: String,
}
