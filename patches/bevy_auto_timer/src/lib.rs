// Copyright 2025 Trung Do <dothanhtrung@pm.me>

//! ### bevy_auto_timer
//!
//! A convenient timer which ticks automatically.

use bevy::prelude::{
    App,
    Commands,
    Component,
    Entity,
    EntityEvent,
    IntoScheduleConfigs,
    Plugin,
    Query,
    Res,
    States,
    Time,
    Timer,
    TimerMode,
    Update,
    in_state,
};
use std::time::Duration;

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

    pub fn any() -> Self {
        Self::new(Vec::new())
    }
}

/// Fake state to trick plugin work in any state.
/// You may not need this.
#[derive(States, Clone, Debug, Hash, Eq, PartialEq)]
pub enum DummyState {}

/// Use this if you don't care to state and want this plugin's systems run all the time.
/// ```rust
/// use bevy::prelude::App;
/// use bevy_auto_timer::AutoTimerPluginAnyState;
/// fn main() {
///     App::new().add_plugins(AutoTimerPluginAnyState::any());
/// }
/// ```
#[derive(Default)]
pub struct AutoTimerPluginAnyState;

impl AutoTimerPluginAnyState {
    pub fn any() -> AutoTimerPlugin<DummyState> {
        AutoTimerPlugin::new(Vec::new())
    }
}

#[derive(Default)]
pub enum ActionOnFinish {
    #[default]
    /// Do nothing
    Nothing,
    /// Despawn entity
    Despawn,
    /// Remove `AutoTimer` out of entity
    Remove,
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

impl AutoTimer {
    pub fn new(duration: Duration, mode: TimerMode) -> Self {
        Self {
            timer: Timer::new(duration, mode),
            ..Self::default()
        }
    }

    pub fn from_seconds(duration: f32, mode: TimerMode) -> Self {
        Self {
            timer: Timer::from_seconds(duration, mode),
            ..Self::default()
        }
    }

    /// Progress in percentage
    pub fn progress(&self) -> f32 {
        self.timer.elapsed().as_secs_f32() / self.timer.duration().as_secs_f32()
    }
}

fn auto_tick(mut commands: Commands, time: Res<Time>, mut query: Query<(&mut AutoTimer, Entity)>) {
    for (mut timer, e) in query.iter_mut() {
        timer.timer.tick(time.delta());
        if timer.timer.just_finished() {
            commands.trigger(AutoTimerFinished { entity: e });
            match timer.action_on_finish {
                ActionOnFinish::Nothing => {}
                ActionOnFinish::Despawn => {
                    commands.entity(e).despawn();
                }
                ActionOnFinish::Remove => {
                    commands.entity(e).remove::<AutoTimer>();
                }
            }
        }
    }
}
