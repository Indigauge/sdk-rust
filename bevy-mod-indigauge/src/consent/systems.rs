use bevy::prelude::*;

#[cfg(feature = "consent")]
use bevy::{picking::hover::Hovered, ui::Pressed, ui_widgets::Button};

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
type AcceptConsentInteractionQuery<'w, 's> =
  Query<'w, 's, (&'static Hovered, Has<Pressed>, &'static mut BackgroundColor), With<AcceptConsentButton>>;

#[cfg(feature = "consent")]
type DeclineConsentInteractionQuery<'w, 's> =
  Query<'w, 's, (&'static Hovered, Has<Pressed>, &'static mut BackgroundColor), With<DeclineConsentButton>>;

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

  let align_items = props.spawn_position.align_items();
  let justify_content = props.spawn_position.justify_content();
  let margin = props.position_margin;
  let title = props.title.clone();
  let message = props.message.clone();
  let decline_button_text = props.decline_button_text.clone();
  let accept_button_text = props.accept_button_text.clone();
  let overlay = styles.overlay;
  let background = styles.background;
  let border = styles.border;
  let text_primary = styles.text_primary;
  let text_secondary = styles.text_secondary;
  let decline_button = styles.decline_button;
  let accept_button = styles.accept_button;

  commands.spawn_scene(bsn! {
      ConsentModal
      Node {
        width: Val::Percent(100.0),
        height: Val::Percent(100.0),
        align_items: align_items,
        justify_content: justify_content,
        margin: margin,
      }
      BackgroundColor(overlay)
      Children [
      Node {
        width: Val::Px(500.0),
        max_width: Val::Percent(95.0),
        padding: UiRect::all(Val::Px(24.0)),
        border: UiRect::all(Val::Px(2.0)),
        row_gap: Val::Px(14.0),
        flex_direction: FlexDirection::Column,
        border_radius: BorderRadius::all(Val::Px(10.0)),
      }
      BackgroundColor(background)
      BorderColor::all(border)
      Children [
        Text::new(title)
        TextFont {
          font_size: FontSize::Px(22.0),
        }
        TextColor(text_primary)
        --
        Text::new(message)
        TextFont {
          font_size: FontSize::Px(16.0),
        }
        TextColor(text_secondary)
        --
      Node {
        width: Val::Percent(100.0),
        justify_content: JustifyContent::End,
        column_gap: Val::Px(10.0),
      }
      Children [
      Node {
        border: UiRect::all(Val::Px(2.0)),
        padding: UiRect::axes(Val::Px(16.0), Val::Px(8.0)),
        border_radius: BorderRadius::all(Val::Px(8.0)),
      }
      Button
      Hovered::default()
      DeclineConsentButton
      BackgroundColor(decline_button)
      BorderColor::all(border)
      Children [
        Text::new(decline_button_text)
        TextFont {
          font_size: FontSize::Px(15.0),
        }
        TextColor(text_primary)
      ]
      --
      Node {
        border: UiRect::all(Val::Px(2.0)),
        padding: UiRect::axes(Val::Px(16.0), Val::Px(8.0)),
        border_radius: BorderRadius::all(Val::Px(8.0)),
      }
      Button
      Hovered::default()
      AcceptConsentButton
      BackgroundColor(accept_button)
      BorderColor::all(border)
      Children [
        Text::new(accept_button_text)
        TextFont {
          font_size: FontSize::Px(15.0),
        }
        TextColor(text_primary)
      ]
      ]
      ]
      ]
  });
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
  for (hovered, pressed, mut background) in &mut query {
    background.0 = if hovered.get() {
      styles.accept_button_hover
    } else {
      styles.accept_button
    };
    if pressed {
      consent_state.choice = IndigaugeConsentChoice::Accepted;
      consent_state.persist_to_disk = props.as_ref().is_none_or(|p| p.persist_choice);
      commands.remove_resource::<ConsentModalProps>();
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
  for (hovered, pressed, mut background) in &mut query {
    background.0 = if hovered.get() {
      styles.decline_button_hover
    } else {
      styles.decline_button
    };
    if pressed {
      consent_state.choice = IndigaugeConsentChoice::Declined;
      consent_state.persist_to_disk = props.as_ref().is_none_or(|p| p.persist_choice);
      commands.remove_resource::<ConsentModalProps>();
    }
  }
}

#[cfg(all(test, feature = "consent"))]
mod tests {
  use super::*;

  fn consent_app() -> App {
    let mut app = App::new();
    app
      .init_resource::<ConsentModalStyles>()
      .init_resource::<IndigaugeConsentState>()
      .insert_resource(ConsentModalProps {
        persist_choice: false,
        ..default()
      })
      .add_systems(Update, (observe_accept_consent_click, observe_decline_consent_click));
    app
  }

  #[test]
  fn consent_scene_spawns_custom_labels_and_functional_buttons() {
    let mut app = consent_app();
    app
      .add_plugins((MinimalPlugins, AssetPlugin::default(), bevy::scene::ScenePlugin))
      .init_asset::<Font>()
      .insert_resource(ConsentModalProps {
        title: "Telemetry consent".to_string(),
        message: "Share anonymous usage data?".to_string(),
        accept_button_text: "Allow".to_string(),
        decline_button_text: "Decline".to_string(),
        persist_choice: false,
        ..default()
      })
      .add_systems(Startup, spawn_consent_modal_ui);
    app.update();

    let mut roots = app.world_mut().query_filtered::<Entity, With<ConsentModal>>();
    let root = roots.single(app.world()).unwrap();
    let mut labels = app.world_mut().query::<&Text>();
    let text: Vec<_> = labels.iter(app.world()).map(|label| label.0.as_str()).collect();
    assert_eq!(text, ["Telemetry consent", "Share anonymous usage data?", "Decline", "Allow"]);
    let mut buttons = app
      .world_mut()
      .query_filtered::<Entity, (With<Button>, With<Hovered>)>();
    assert_eq!(buttons.iter(app.world()).count(), 2);
    let mut accept = app.world_mut().query_filtered::<Entity, With<AcceptConsentButton>>();
    let accept = accept.single(app.world()).unwrap();
    app.world_mut().entity_mut(accept).insert(Pressed);
    app.update();
    assert_eq!(app.world().resource::<IndigaugeConsentState>().choice, IndigaugeConsentChoice::Accepted);
    app.add_systems(Update, despawn_consent_modal_ui);
    app.update();
    assert!(app.world().get_entity(root).is_err());
    assert!(app.world().get_entity(accept).is_err());
  }

  #[test]
  fn consent_hover_style_resets_when_pointer_leaves() {
    let mut app = consent_app();
    let button = app
      .world_mut()
      .spawn((AcceptConsentButton, Hovered(true), BackgroundColor::default()))
      .id();
    app.update();
    assert_eq!(
      app.world().get::<BackgroundColor>(button).unwrap().0,
      app.world().resource::<ConsentModalStyles>().accept_button_hover,
    );
    app.world_mut().entity_mut(button).insert(Hovered(false));
    app.update();
    assert_eq!(
      app.world().get::<BackgroundColor>(button).unwrap().0,
      app.world().resource::<ConsentModalStyles>().accept_button,
    );
    assert_eq!(app.world().resource::<IndigaugeConsentState>().choice, IndigaugeConsentChoice::Unknown);
  }

  #[test]
  fn pressed_accept_button_selects_consent() {
    let mut app = consent_app();
    app
      .world_mut()
      .spawn((AcceptConsentButton, Hovered(true), Pressed, BackgroundColor::default()));
    app.update();
    let state = app.world().resource::<IndigaugeConsentState>();
    assert_eq!(state.choice, IndigaugeConsentChoice::Accepted);
    assert!(!state.persist_to_disk);
    assert!(!app.world().contains_resource::<ConsentModalProps>());
  }

  #[test]
  fn pressed_decline_button_selects_consent() {
    let mut app = consent_app();
    app
      .world_mut()
      .spawn((DeclineConsentButton, Hovered(true), Pressed, BackgroundColor::default()));
    app.update();
    let state = app.world().resource::<IndigaugeConsentState>();
    assert_eq!(state.choice, IndigaugeConsentChoice::Declined);
    assert!(!state.persist_to_disk);
    assert!(!app.world().contains_resource::<ConsentModalProps>());
  }
}
