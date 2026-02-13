use indigauge_types::prelude::EventPayload;

/// Queued event with basic validation helpers.
#[derive(Clone, Debug)]
pub struct QueuedEvent {
  payload: EventPayload,
}

impl QueuedEvent {
  pub fn new(payload: EventPayload) -> Self {
    Self { payload }
  }

  pub fn into_inner(self) -> EventPayload {
    self.payload
  }

  pub fn validate(&self) -> Result<(), String> {
    let (ns, t) = self.payload.event_type.split_once('.').ok_or("Invalid event type")?;
    if ns.trim().is_empty() || t.trim().is_empty() {
      return Err("Invalid event type".to_string());
    }
    Ok(())
  }
}

/// Runtime validation for event types.
pub const fn validate_event_type(s: &str) -> Result<(), &'static str> {
  let bytes = s.as_bytes();
  let len = bytes.len();

  if len < 3 {
    return Err("Invalid event type: too short (expected 'a.b')");
  }

  let mut dot_index: Option<usize> = None;
  let mut i = 0;

  while i < len {
    let b = bytes[i];
    match b {
      b'a'..=b'z' | b'A'..=b'Z' => {},
      b'.' => {
        if dot_index.is_some() {
          return Err("Invalid event type: multiple '.' found");
        }
        dot_index = Some(i);
      },
      _ => return Err("Invalid event type: only letters and a single '.' are allowed"),
    }
    i += 1;
  }

  let dot_pos = match dot_index {
    Some(p) => p,
    None => return Err("Invalid event type: must contain one '.'"),
  };

  if dot_pos == 0 || dot_pos == len - 1 {
    return Err("Invalid event type: '.' cannot be the first or last character");
  }

  Ok(())
}

/// Compile-time validation for event types used in macros.
pub const fn validate_event_type_compile_time(s: &str) -> &str {
  if let Err(err) = validate_event_type(s) {
    panic!("{}", err);
  }
  s
}

#[cfg(test)]
mod tests {
  use super::*;

  #[test]
  fn validates_event_type() {
    assert!(validate_event_type("game.start").is_ok());
    assert!(validate_event_type("game").is_err());
    assert!(validate_event_type(".start").is_err());
    assert!(validate_event_type("game.").is_err());
    assert!(validate_event_type("game..start").is_err());
  }
}
