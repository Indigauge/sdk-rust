use bevy::prelude::*;

#[cfg(not(target_family = "wasm"))]
use std::fs;

use crate::{
  config::BevyIndigaugeConfig,
  consent::{
    events::IndigaugeConsentSelectedEvent,
    resources::{IndigaugeConsentChoice, IndigaugeConsentState},
  },
  event::resources::{BufferedEvents, EventQueueReceiver},
  session::resources::SessionApiKey,
};
use indigauge_core::state::set_telemetry_consent;

#[cfg(not(target_family = "wasm"))]
use crate::consent::utils::consent_file_path;

#[cfg(feature = "consent")]
use crate::consent::{
  components::{AcceptConsentButton, ConsentModal, DeclineConsentButton},
  resources::{ConsentModalProps, ConsentModalStyles},
};

#[cfg(feature = "consent")]
type AcceptConsentInteractionQuery<'w, 's> = Query<
  'w,
  's,
  (&'static Interaction, &'static mut BackgroundColor),
  (With<AcceptConsentButton>, Changed<Interaction>),
>;

#[cfg(feature = "consent")]
type DeclineConsentInteractionQuery<'w, 's> = Query<
  'w,
  's,
  (&'static Interaction, &'static mut BackgroundColor),
  (With<DeclineConsentButton>, Changed<Interaction>),
>;

/// Loads persisted telemetry consent from the user's preference folder.
#[cfg(not(target_family = "wasm"))]
pub fn load_persisted_consent(config: Res<BevyIndigaugeConfig>, mut consent_state: ResMut<IndigaugeConsentState>) {
  let Some(path) = consent_file_path(config.game_name()) else {
    return;
  };

  let Ok(value) = fs::read_to_string(path) else {
    return;
  };

  if let Some(choice) = IndigaugeConsentChoice::from_disk_value(&value) {
    consent_state.choice = choice;
    consent_state.persist_to_disk = true;
  }
}

/// Loads persisted telemetry consent from the user's preference folder.
#[cfg(target_family = "wasm")]
pub fn load_persisted_consent(_config: Res<BevyIndigaugeConfig>, _consent_state: ResMut<IndigaugeConsentState>) {}

/// Persists telemetry consent choice to the user's preference folder.
#[cfg(not(target_family = "wasm"))]
pub fn persist_consent_choice(config: Res<BevyIndigaugeConfig>, consent_state: Res<IndigaugeConsentState>) {
  if !consent_state.persist_to_disk {
    return;
  }

  let Some(path) = consent_file_path(config.game_name()) else {
    return;
  };

  match consent_state.choice.as_disk_value() {
    Some(value) => {
      if let Some(parent) = path.parent() {
        let _ = fs::create_dir_all(parent);
      }
      let _ = fs::write(path, value);
    },
    None => {
      let _ = fs::remove_file(path);
    },
  }
}

/// Persists telemetry consent choice to the user's preference folder.
#[cfg(target_family = "wasm")]
pub fn persist_consent_choice(_config: Res<BevyIndigaugeConfig>, _consent_state: Res<IndigaugeConsentState>) {}

/// Synchronizes current consent state into indigauge-core global consent gate.
pub fn sync_core_consent_flag(consent_state: Res<IndigaugeConsentState>) {
  set_telemetry_consent(consent_state.choice == IndigaugeConsentChoice::Accepted);
}

/// Revokes active telemetry session and clears queued/buffered events when consent is not accepted.
pub fn enforce_consent_gate(
  mut commands: Commands,
  consent_state: Res<IndigaugeConsentState>,
  session_key: Option<Res<SessionApiKey>>,
  mut buffered_events: ResMut<BufferedEvents>,
  mut queue_receiver: Option<ResMut<EventQueueReceiver>>,
) {
  if consent_state.choice == IndigaugeConsentChoice::Accepted {
    return;
  }

  if session_key.is_some() {
    commands.remove_resource::<SessionApiKey>();
  }

  buffered_events.events.clear();

  if let Some(receiver) = queue_receiver.as_mut() {
    while receiver.try_recv().is_ok() {}
  }
}

/// Emits a consent-selected event whenever the consent state transitions to a known choice.
pub fn emit_consent_selected_event(mut commands: Commands, consent_state: Res<IndigaugeConsentState>) {
  match consent_state.choice {
    IndigaugeConsentChoice::Accepted | IndigaugeConsentChoice::Declined => {
      commands.trigger(IndigaugeConsentSelectedEvent {
        choice: consent_state.choice,
      });
    },
    IndigaugeConsentChoice::Unknown => {},
  }
}

#[cfg(feature = "consent")]
/// Ensures modal props are removed when consent was already decided previously.
pub fn dismiss_modal_if_choice_known(mut commands: Commands, consent_state: Res<IndigaugeConsentState>) {
  if consent_state.choice != IndigaugeConsentChoice::Unknown {
    commands.remove_resource::<ConsentModalProps>();
  }
}

#[cfg(feature = "consent")]
/// Spawns or respawns the consent modal UI hierarchy.
pub fn spawn_consent_modal_ui(
  mut commands: Commands,
  styles: Res<ConsentModalStyles>,
  props: Res<ConsentModalProps>,
  modal_query: Query<Entity, With<ConsentModal>>,
) {
  if let Ok(existing_modal) = modal_query.single() {
    commands.entity(existing_modal).despawn();
  }

  let root = commands
    .spawn((
      Node {
        width: Val::Percent(100.0),
        height: Val::Percent(100.0),
        align_items: props.spawn_position.align_items(),
        justify_content: props.spawn_position.justify_content(),
        margin: props.position_margin,
        ..default()
      },
      BackgroundColor(styles.overlay),
      ConsentModal,
    ))
    .id();

  let panel = commands
    .spawn((
      ChildOf(root),
      Node {
        width: Val::Px(500.0),
        max_width: Val::Percent(95.0),
        padding: UiRect::all(Val::Px(24.0)),
        border: UiRect::all(Val::Px(2.0)),
        row_gap: Val::Px(14.0),
        flex_direction: FlexDirection::Column,
        border_radius: BorderRadius::all(Val::Px(10.0)),
        ..default()
      },
      BackgroundColor(styles.background),
      BorderColor::all(styles.border),
    ))
    .id();

  commands.spawn((
    ChildOf(panel),
    Text::new(props.title.clone()),
    TextFont::from_font_size(22.0),
    TextColor(styles.text_primary),
  ));

  commands.spawn((
    ChildOf(panel),
    Text::new(props.message.clone()),
    TextFont::from_font_size(16.0),
    TextColor(styles.text_secondary),
  ));

  let actions = commands
    .spawn((
      ChildOf(panel),
      Node {
        width: Val::Percent(100.0),
        justify_content: JustifyContent::End,
        column_gap: Val::Px(10.0),
        ..default()
      },
    ))
    .id();

  let decline = commands
    .spawn((
      ChildOf(actions),
      Node {
        border: UiRect::all(Val::Px(2.0)),
        padding: UiRect::axes(Val::Px(16.0), Val::Px(8.0)),
        border_radius: BorderRadius::all(Val::Px(8.0)),
        ..default()
      },
      Button,
      DeclineConsentButton,
      BackgroundColor(styles.decline_button),
      BorderColor::all(styles.border),
    ))
    .id();
  commands.spawn((
    ChildOf(decline),
    Text::new(props.decline_button_text.clone()),
    TextFont::from_font_size(15.0),
    TextColor(styles.text_primary),
  ));

  let accept = commands
    .spawn((
      ChildOf(actions),
      Node {
        border: UiRect::all(Val::Px(2.0)),
        padding: UiRect::axes(Val::Px(16.0), Val::Px(8.0)),
        border_radius: BorderRadius::all(Val::Px(8.0)),
        ..default()
      },
      Button,
      AcceptConsentButton,
      BackgroundColor(styles.accept_button),
      BorderColor::all(styles.border),
    ))
    .id();
  commands.spawn((
    ChildOf(accept),
    Text::new(props.accept_button_text.clone()),
    TextFont::from_font_size(15.0),
    TextColor(styles.text_primary),
  ));
}

#[cfg(feature = "consent")]
/// Despawns the active consent modal UI tree.
pub fn despawn_consent_modal_ui(mut commands: Commands, query: Query<Entity, With<ConsentModal>>) {
  for entity in &query {
    commands.entity(entity).despawn();
  }
}

#[cfg(feature = "consent")]
/// Synchronizes modal display with the `visible` flag in props.
pub fn consent_modal_visibility_sync(props: Res<ConsentModalProps>, mut query: Query<&mut Node, With<ConsentModal>>) {
  if let Ok(mut root_node) = query.single_mut() {
    root_node.display = if props.visible { Display::Flex } else { Display::None };
  }
}

#[cfg(feature = "consent")]
/// Handles accept button interactions and writes consent state.
pub fn observe_accept_consent_click(
  mut commands: Commands,
  props: Option<Res<ConsentModalProps>>,
  styles: Res<ConsentModalStyles>,
  mut consent_state: ResMut<IndigaugeConsentState>,
  mut query: AcceptConsentInteractionQuery,
) {
  for (interaction, mut background) in &mut query {
    match *interaction {
      Interaction::Hovered => background.0 = styles.accept_button_hover,
      Interaction::None => background.0 = styles.accept_button,
      Interaction::Pressed => {
        consent_state.choice = IndigaugeConsentChoice::Accepted;
        consent_state.persist_to_disk = props.as_ref().is_none_or(|p| p.persist_choice);
        commands.remove_resource::<ConsentModalProps>();
      },
    }
  }
}

#[cfg(feature = "consent")]
/// Handles decline button interactions and writes consent state.
pub fn observe_decline_consent_click(
  mut commands: Commands,
  props: Option<Res<ConsentModalProps>>,
  styles: Res<ConsentModalStyles>,
  mut consent_state: ResMut<IndigaugeConsentState>,
  mut query: DeclineConsentInteractionQuery,
) {
  for (interaction, mut background) in &mut query {
    match *interaction {
      Interaction::Hovered => background.0 = styles.decline_button_hover,
      Interaction::None => background.0 = styles.decline_button,
      Interaction::Pressed => {
        consent_state.choice = IndigaugeConsentChoice::Declined;
        consent_state.persist_to_disk = props.as_ref().is_none_or(|p| p.persist_choice);
        commands.remove_resource::<ConsentModalProps>();
      },
    }
  }
}
