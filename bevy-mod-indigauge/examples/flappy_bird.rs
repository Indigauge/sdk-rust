use std::env;

use bevy::prelude::*;
use bevy_mod_indigauge::prelude::*;
use serde::Serialize;

const WORLD_LEFT: f32 = -460.0;
const WORLD_RIGHT: f32 = 460.0;
const WORLD_TOP: f32 = 300.0;
const WORLD_BOTTOM: f32 = -300.0;

const BIRD_X: f32 = -180.0;
const BIRD_SIZE: Vec2 = Vec2::new(36.0, 28.0);
const GRAVITY: f32 = 980.0;
const FLAP_IMPULSE: f32 = 360.0;

const PIPE_WIDTH: f32 = 70.0;
const PIPE_HEIGHT: f32 = 420.0;
const PIPE_GAP: f32 = 180.0;
const PIPE_SPEED: f32 = 240.0;
const PIPE_SPAWN_SECONDS: f32 = 1.45;

const SKY_COLOR: Color = Color::srgb(0.58, 0.82, 0.95);
const BIRD_COLOR: Color = Color::srgb(0.98, 0.88, 0.23);
const PIPE_COLOR: Color = Color::srgb(0.18, 0.62, 0.24);
const TEXT_COLOR: Color = Color::srgb(0.11, 0.17, 0.22);
const GAME_OVER_COLOR: Color = Color::srgb(0.74, 0.09, 0.09);

struct EventType;

impl EventType {
  const PLAYER_FLAP: &'static str = "player.flap";
  const PLAYER_CRASH: &'static str = "player.crash";
  const ROUND_START: &'static str = "round.start";
}

#[derive(Clone, Copy, PartialEq, Eq, Hash, Debug, Default, States)]
enum GameState {
  #[default]
  InitializeSession,
  Setup,
  Playing,
  Paused,
  GameOver,
}

#[derive(Component)]
struct Bird;

#[derive(Component)]
struct Pipe;

#[derive(Component)]
struct GameOverText;

#[derive(Component)]
struct ScoreboardUi;

#[derive(Component, Deref, DerefMut)]
struct BirdVelocity(f32);

#[derive(Component)]
struct ScoreTrigger {
  passed: bool,
}

#[derive(Resource, Default, Deref, DerefMut)]
struct Score(u32);

#[derive(Resource)]
struct SpawnTimer(Timer);

#[derive(Resource, Default)]
struct PipeSequence {
  index: usize,
}

#[derive(Resource, Deref, DerefMut, Default, Serialize)]
pub struct FlappySessionMetadata {
  pub score: u32,
}

fn main() {
  App::new()
    .add_plugins(DefaultPlugins)
    .insert_state(GameState::default())
    .insert_resource(ClearColor(SKY_COLOR))
    .insert_resource(Score::default())
    .insert_resource(SpawnTimer(Timer::from_seconds(PIPE_SPAWN_SECONDS, TimerMode::Repeating)))
    .insert_resource(PipeSequence::default())
    .insert_resource(FlappySessionMetadata::default())
    .add_plugins(
      IndigaugePlugin::<FlappySessionMetadata>::new("YOUR_PUBLIC_KEY", "Flappy Bird", env!("CARGO_PKG_VERSION"))
        .mode(IndigaugeMode::Dev)
        .log_level(IndigaugeLogLevel::Info),
    )
    .add_systems(Startup, setup)
    .add_systems(OnEnter(GameState::InitializeSession), start_default_session)
    .add_observer(switch_state_after_session_init(GameState::Setup))
    // Switch to paused state when feedback is spawned
    .add_observer(switch_state_on_feedback_spawn(GameState::Paused))
    // Switch back to playing state when feedback is despawned
    .add_observer(switch_state_on_feedback_despawn(GameState::Playing))
    .add_systems(Update, reset_round.run_if(in_state(GameState::Setup)))
    .add_systems(OnEnter(GameState::GameOver), spawn_game_over_text)
    .add_systems(
      Update,
      (
        trigger_feedback,
        flap_input,
        apply_gravity,
        move_obstacles,
        despawn_offscreen_obstacles,
        spawn_pipes,
        score_pipes,
        detect_collisions,
        update_scoreboard,
      )
        .run_if(in_state(GameState::Playing)),
    )
    .add_systems(Update, restart_after_game_over.run_if(in_state(GameState::GameOver)))
    .run();
}

fn setup(mut commands: Commands) {
  commands.spawn((Camera2d, IsDefaultUiCamera));

  commands.spawn((
    Sprite::from_color(BIRD_COLOR, Vec2::ONE),
    Transform {
      translation: Vec3::new(BIRD_X, 0.0, 1.0),
      scale: BIRD_SIZE.extend(1.0),
      ..default()
    },
    Bird,
    BirdVelocity(0.0),
  ));

  commands
    .spawn((
      Text::new("Score: "),
      TextColor(TEXT_COLOR),
      ScoreboardUi,
      Node {
        position_type: PositionType::Absolute,
        left: Val::Px(14.0),
        top: Val::Px(14.0),
        ..default()
      },
    ))
    .with_child((TextSpan::new("0"), TextColor(TEXT_COLOR)));

  commands.spawn((
    Text::new("SPACE flap | R restart | F2 default feedback | F3 bug report"),
    TextColor(TEXT_COLOR),
    Node {
      position_type: PositionType::Absolute,
      left: Val::Px(14.0),
      bottom: Val::Px(12.0),
      ..default()
    },
  ));
}

fn reset_round(
  mut commands: Commands,
  mut score: ResMut<Score>,
  mut timer: ResMut<SpawnTimer>,
  mut sequence: ResMut<PipeSequence>,
  bird_query: Single<(&mut Transform, &mut BirdVelocity), With<Bird>>,
  pipes: Query<Entity, With<Pipe>>,
  triggers: Query<Entity, With<ScoreTrigger>>,
  game_over_texts: Query<Entity, With<GameOverText>>,
  mut next_state: ResMut<NextState<GameState>>,
) {
  for entity in &pipes {
    commands.entity(entity).despawn();
  }

  for entity in &triggers {
    commands.entity(entity).despawn();
  }

  for entity in &game_over_texts {
    commands.entity(entity).despawn();
  }

  **score = 0;
  timer.0.reset();
  sequence.index = 0;

  let (mut bird_transform, mut velocity) = bird_query.into_inner();
  bird_transform.translation = Vec3::new(BIRD_X, 0.0, 1.0);
  **velocity = 0.0;

  ig_info!(EventType::ROUND_START);
  next_state.set(GameState::Playing);
}

fn flap_input(keys: Res<ButtonInput<KeyCode>>, bird_query: Single<(&Transform, &mut BirdVelocity), With<Bird>>) {
  if keys.just_pressed(KeyCode::Space) {
    let (bird_transform, mut velocity) = bird_query.into_inner();
    **velocity = FLAP_IMPULSE;
    ig_info!(EventType::PLAYER_FLAP, { "bird_y": bird_transform.translation.y });
  }
}

fn apply_gravity(bird_query: Single<(&mut Transform, &mut BirdVelocity), With<Bird>>, time: Res<Time>) {
  let (mut bird_transform, mut velocity) = bird_query.into_inner();
  **velocity -= GRAVITY * time.delta_secs();
  bird_transform.translation.y += **velocity * time.delta_secs();
}

fn move_obstacles(mut obstacles: Query<&mut Transform, Or<(With<Pipe>, With<ScoreTrigger>)>>, time: Res<Time>) {
  let delta = PIPE_SPEED * time.delta_secs();
  for mut transform in &mut obstacles {
    transform.translation.x -= delta;
  }
}

fn despawn_offscreen_obstacles(
  mut commands: Commands,
  obstacles: Query<(Entity, &Transform), Or<(With<Pipe>, With<ScoreTrigger>)>>,
) {
  for (entity, transform) in &obstacles {
    if transform.translation.x < WORLD_LEFT - PIPE_WIDTH {
      commands.entity(entity).despawn();
    }
  }
}

fn spawn_pipes(
  mut commands: Commands,
  mut timer: ResMut<SpawnTimer>,
  mut sequence: ResMut<PipeSequence>,
  time: Res<Time>,
) {
  if !timer.0.tick(time.delta()).just_finished() {
    return;
  }

  let gap_centers = [-130.0, -70.0, -20.0, 40.0, 110.0];
  let gap_center = gap_centers[sequence.index % gap_centers.len()];
  sequence.index += 1;

  let x = WORLD_RIGHT + PIPE_WIDTH;
  let top_pipe_y = gap_center + PIPE_GAP * 0.5 + PIPE_HEIGHT * 0.5;
  let bottom_pipe_y = gap_center - PIPE_GAP * 0.5 - PIPE_HEIGHT * 0.5;

  for y in [top_pipe_y, bottom_pipe_y] {
    commands.spawn((
      Sprite::from_color(PIPE_COLOR, Vec2::ONE),
      Transform {
        translation: Vec3::new(x, y, 0.0),
        scale: Vec3::new(PIPE_WIDTH, PIPE_HEIGHT, 1.0),
        ..default()
      },
      Pipe,
    ));
  }

  commands.spawn((Transform::from_translation(Vec3::new(x, gap_center, 0.0)), ScoreTrigger { passed: false }));
}

fn score_pipes(
  mut score: ResMut<Score>,
  mut metadata: ResMut<FlappySessionMetadata>,
  mut triggers: Query<(&Transform, &mut ScoreTrigger)>,
) {
  for (transform, mut trigger) in &mut triggers {
    if !trigger.passed && transform.translation.x < BIRD_X {
      trigger.passed = true;
      **score += 1;
      metadata.score = **score;
    }
  }
}

fn detect_collisions(
  bird_query: Single<&Transform, With<Bird>>,
  pipes: Query<&Transform, With<Pipe>>,
  mut next_state: ResMut<NextState<GameState>>,
) {
  let bird = bird_query.into_inner().translation;
  let bird_half = BIRD_SIZE * 0.5;

  if bird.y + bird_half.y >= WORLD_TOP || bird.y - bird_half.y <= WORLD_BOTTOM {
    ig_info!(EventType::PLAYER_CRASH, { "reason": "out_of_bounds", "bird_y": bird.y });
    next_state.set(GameState::GameOver);
    return;
  }

  for pipe_transform in &pipes {
    let pipe_center = pipe_transform.translation;
    let pipe_half = Vec2::new(PIPE_WIDTH * 0.5, PIPE_HEIGHT * 0.5);

    let overlaps_x = (bird.x - pipe_center.x).abs() < bird_half.x + pipe_half.x;
    let overlaps_y = (bird.y - pipe_center.y).abs() < bird_half.y + pipe_half.y;

    if overlaps_x && overlaps_y {
      ig_info!(EventType::PLAYER_CRASH, { "reason": "hit_pipe", "bird_y": bird.y });
      next_state.set(GameState::GameOver);
      return;
    }
  }
}

fn update_scoreboard(
  score: Res<Score>,
  score_root: Single<Entity, (With<ScoreboardUi>, With<Text>)>,
  mut writer: TextUiWriter,
) {
  *writer.text(*score_root, 1) = score.to_string();
}

fn spawn_game_over_text(mut commands: Commands, score: Res<Score>) {
  commands.spawn((
    Text::new(format!("Game Over! Score: {}. Press R to restart.", **score)),
    TextColor(GAME_OVER_COLOR),
    Node {
      position_type: PositionType::Absolute,
      top: Val::Percent(46.0),
      left: Val::Percent(20.0),
      ..default()
    },
    GameOverText,
  ));
}

fn restart_after_game_over(keys: Res<ButtonInput<KeyCode>>, mut next_state: ResMut<NextState<GameState>>) {
  if keys.just_pressed(KeyCode::KeyR) {
    next_state.set(GameState::Setup);
  }
}

fn trigger_feedback(
  mut commands: Commands,
  keys: Res<ButtonInput<KeyCode>>,
  existing: Option<Res<FeedbackPanelProps>>,
) {
  if existing.is_some() {
    return;
  }

  if keys.just_pressed(KeyCode::F3) {
    commands.insert_resource(
      FeedbackPanelProps::with_question("What made this run feel unfair?", FeedbackCategory::Gameplay)
        .title("Flappy Bird Feedback")
        .allow_screenshot(true),
    );
  }
}
