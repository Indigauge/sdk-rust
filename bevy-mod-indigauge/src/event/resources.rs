use std::ops::{Deref, DerefMut};

use bevy::prelude::*;
use crossbeam_channel::Receiver;

use indigauge_core::event::QueuedEvent;

#[derive(Resource)]
pub struct EventQueueReceiver {
  rx: Receiver<QueuedEvent>,
}

impl EventQueueReceiver {
  pub fn new(rx: Receiver<QueuedEvent>) -> Self {
    Self { rx }
  }
}

impl Deref for EventQueueReceiver {
  type Target = Receiver<QueuedEvent>;

  fn deref(&self) -> &Self::Target {
    &self.rx
  }
}

impl DerefMut for EventQueueReceiver {
  fn deref_mut(&mut self) -> &mut Self::Target {
    &mut self.rx
  }
}

#[derive(Resource, Default)]
pub struct BufferedEvents {
  pub events: Vec<QueuedEvent>,
}
