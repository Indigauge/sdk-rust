use bevy::prelude::*;
use serde::Serialize;
use std::{marker::PhantomData, ops::Deref};

/// Resource holding the active session API key/token.
#[derive(Resource)]
pub struct SessionApiKey {
  key: String,
}

impl SessionApiKey {
  pub(crate) fn new(key: impl Into<String>) -> Self {
    Self { key: key.into() }
  }
}

impl Deref for SessionApiKey {
  type Target = String;

  fn deref(&self) -> &Self::Target {
    &self.key
  }
}

/// Marker metadata type for integrations that do not use session metadata.
#[derive(Resource, Serialize)]
pub struct EmptySessionMeta;

/// Internal resource that tracks whether session metadata needs syncing.
#[derive(Resource)]
pub struct SessionMeta<T: Resource + Serialize> {
  pub(crate) is_changed: bool,
  data: PhantomData<T>,
}

impl<T> Default for SessionMeta<T>
where
  T: Resource + Serialize,
{
  fn default() -> Self {
    Self {
      is_changed: false,
      data: PhantomData,
    }
  }
}
