// Copyright 2025 Trung Do <dothanhtrung@pm.me>

//! ### bevy_auto_timer
//!
//! A convenient timer which ticks automatically.

use bevy::prelude::{
  App, Commands, Component, Entity, EntityEvent, IntoScheduleConfigs, Plugin, Query, Res, States, Time, Timer, Update,
  in_state,
};

macro_rules! plugin_systems {
  ( ) => {
    (auto_tick)
  };
}

/// The main plugin
#[derive(Default)]
pub struct AutoTimerPlugin<T>
where
  T: States,
{
  /// List of game state this plugin will run in
  pub states: Vec<T>,
}

impl<T> Plugin for AutoTimerPlugin<T>
where
  T: States,
{
  fn build(&self, app: &mut App) {
    if self.states.is_empty() {
      app.add_systems(Update, plugin_systems!());
    } else {
      for state in self.states.iter() {
        app.add_systems(Update, plugin_systems!().run_if(in_state(state.clone())));
      }
    }
  }
}

impl<T> AutoTimerPlugin<T>
where
  T: States,
{
  pub fn new(states: Vec<T>) -> Self {
    Self { states }
  }
}

#[derive(Default)]
pub enum ActionOnFinish {
  #[default]
  /// Do nothing
  Nothing,
}

/// Timer component which ticks automatically
#[derive(Component, Default)]
pub struct AutoTimer {
  pub timer: Timer,
  pub action_on_finish: ActionOnFinish,
}

/// Triggered when the timer is finished
#[derive(EntityEvent)]
pub struct AutoTimerFinished {
  pub entity: Entity,
}

fn auto_tick(mut commands: Commands, time: Res<Time>, mut query: Query<(&mut AutoTimer, Entity)>) {
  for (mut timer, e) in query.iter_mut() {
    timer.timer.tick(time.delta());
    if timer.timer.just_finished() {
      commands.trigger(AutoTimerFinished { entity: e });
      match timer.action_on_finish {
        ActionOnFinish::Nothing => {},
      }
    }
  }
}
