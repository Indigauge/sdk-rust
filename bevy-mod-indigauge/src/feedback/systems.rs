use crate::{
  feedback::observers::{
    observe_cancel_click, observe_category_dropdown_click, observe_category_item_click,
    observe_screenshot_toggle_click, observe_submit_click,
  },
  feedback::{components::*, helpers::*, resources::*, types::FeedbackCategory},
};
use bevy::{
  input::mouse::{MouseScrollUnit, MouseWheel},
  input_focus::tab_navigation::TabIndex,
  picking::{
    cursor::EntityCursor,
    hover::{HoverMap, Hovered},
  },
  prelude::*,
  text::{EditableText, TextCursorStyle},
  ui::Pressed,
  ui_widgets::Button,
  window::SystemCursorIcon,
};
use bevy_feathers::display::label;
use indigauge_core::utils::select;

const LINE_HEIGHT: f32 = 21.;

/// Despawns the active feedback panel UI tree.
pub fn despawn_feedback_panel(mut commands: Commands, query: Query<Entity, With<FeedbackPanel>>) {
  for entity in &query {
    commands.entity(entity).despawn();
  }
}

/// Opens the feedback panel when the configured toggle key is pressed.
pub fn toggle_panel_visibility_with_key(
  mut commands: Commands,
  keys: Res<ButtonInput<KeyCode>>,
  toggle_button: Res<FeedbackKeyCodeToggle>,
) {
  if keys.just_pressed(toggle_button.0) {
    commands.insert_resource(FeedbackPanelProps::default());
  }
}

// Synk display med visible
/// Synchronizes panel node display with the `visible` flag in props.
pub fn panel_visibility_sync(props: Res<FeedbackPanelProps>, mut q: Query<&mut Node, With<FeedbackPanel>>) {
  if let Ok(mut node) = q.single_mut() {
    node.display = select(Display::Flex, Display::None, props.visible);
  }
}

type HoverAndClickInteractionQuery<'a, 'w, 's> = Query<
  'w,
  's,
  (
    &'a Hovered,
    Has<Pressed>,
    Entity,
    &'a mut BackgroundColor,
    &'a mut BorderColor,
    Option<&'a ButtonHoverStyle>,
    Option<&'a ButtonPressedStyle>,
    Option<&'a OriginalButtonStyles>,
    Has<HoldPressed>,
    Has<Active>,
  ),
  Or<(With<ButtonHoverStyle>, With<ButtonPressedStyle>)>,
>;

// Handle hover and click states
/// Applies hover/pressed style transitions to feedback UI buttons.
pub fn handle_hover_and_click_styles(mut commands: Commands, mut q: HoverAndClickInteractionQuery) {
  q.iter_mut().for_each(
    |(hovered, pressed, entity, mut bg_color, mut border_color, bhs, bps, obs, hold_after_press, is_active)| {
      if obs.is_none()
        && let Ok(mut ecm) = commands.get_entity(entity)
      {
        ecm.try_insert_if_new(OriginalButtonStyles {
          background: bg_color.0,
          border: border_color.top,
        });
      }
      if pressed {
        if let Some(pressed_style) = bps {
          bg_color.0 = pressed_style.background;
          *border_color = BorderColor::all(pressed_style.border);
        }

        if hold_after_press && let Ok(mut ecm) = commands.get_entity(entity) {
          ecm.try_insert(Active);
        }
      } else if hovered.get() {
        if !is_active && let Some(hover_style) = bhs {
          bg_color.0 = hover_style.background;
          *border_color = BorderColor::all(hover_style.border);
        }
      } else {
        if !is_active && let Some(original_styles) = obs {
          bg_color.0 = original_styles.background;
          *border_color = BorderColor::all(original_styles.border);
        }
      }
    },
  );
}

// Synk dropdown synlighet (legacy Bevy UI backend)
#[cfg(all(feature = "feedback", not(feature = "feedback_egui")))]
/// Synchronizes category dropdown visibility for the legacy Bevy UI backend.
pub fn dropdown_visibility_sync(
  ui_state: Res<crate::feedback::resources::FeedbackUiState>,
  mut q: Query<&mut Node, With<CategoryList>>,
) {
  if let Ok(mut n) = q.single_mut() {
    n.display = select(Display::Flex, Display::None, ui_state.dropdown_open);
  }
}

/// Updates the scroll position of scrollable nodes in response to mouse input
pub fn update_scroll_position(
  mut mouse_wheel_events: MessageReader<MouseWheel>,
  hover_map: Res<HoverMap>,
  mut scrolled_node_query: Query<&mut ScrollPosition>,
) {
  for mouse_wheel_event in mouse_wheel_events.read() {
    let (dx, dy) = match mouse_wheel_event.unit {
      MouseScrollUnit::Line => (mouse_wheel_event.x * LINE_HEIGHT, mouse_wheel_event.y * LINE_HEIGHT),
      MouseScrollUnit::Pixel => (mouse_wheel_event.x, mouse_wheel_event.y),
    };

    for (_pointer, pointer_map) in hover_map.iter() {
      for entity in pointer_map.keys().copied() {
        if let Ok(mut scroll_position) = scrolled_node_query.get_mut(entity) {
          scroll_position.x -= dx;
          scroll_position.y -= dy;
        }
      }
    }
  }
}

/// Spawns (or respawns) the feedback panel UI hierarchy.
pub fn spawn_feedback_ui(
  mut commands: Commands,
  styles: Res<FeedbackPanelStyles>,
  props: Res<FeedbackPanelProps>,
  mut form: ResMut<FeedbackFormState>,
  feedback_panel_query: Query<Entity, With<FeedbackPanel>>,
) {
  if let Ok(root_entity) = feedback_panel_query.single() {
    commands.entity(root_entity).despawn();
  }

  *form = FeedbackFormState::default();

  if let Some(category) = &props.category
    && let Some(question) = &props.question
  {
    form.category = *category;
    form.question = Some(question.clone());
  }

  let panel_align_items = props.spawn_position.align_items();
  let panel_justify_content = props.spawn_position.justify_content();
  let panel_margin = props.position_margin;
  let panel_title = props.title.clone();
  let panel_question = props.question.clone();
  let has_title = panel_title.is_some();
  let allow_screenshot = props.allow_screenshot;

  let style_primary = styles.primary;
  let style_primary_hover = styles.primary_hover;
  let style_secondary = styles.secondary;
  let style_surface = styles.surface;
  let style_background = styles.background;
  let style_border = styles.border;
  let style_text_primary = styles.text_primary;
  let style_text_secondary = styles.text_secondary;
  let style_error = styles.error;

  let root_entity = commands
    .spawn_scene(bsn! {
      FeedbackPanel
      Node {
        width: Val::Percent(100.0),
        height: Val::Percent(100.0),
        align_items: panel_align_items,
        justify_content: panel_justify_content,
      }
      BackgroundColor(Color::NONE)
    })
    .id();

  let panel_entity = commands
    .spawn_scene(bsn! {
      ChildOf(root_entity)
      FeedbackPanelBody
      @panel(style_background, style_border)
      Node {
        width: Val::Px(420.0),
        min_height: Val::Px(420.0),
        margin: panel_margin,
        padding: UiRect::axes(Val::Px(48.0), Val::Px(32.0)),
        border: UiRect::all(Val::Px(2.0)),
        flex_direction: FlexDirection::Column,
        row_gap: Val::Px(10.0),
        border_radius: BorderRadius::all(Val::Px(8.0)),
      }
    })
    .id();

  if let Some(title) = panel_title {
    commands.spawn_scene(bsn! {
      ChildOf(panel_entity)
      Text::default()
      Node::default()
      Children [
        TextSpan::new(title)
        TextFont {
          font_size: FontSize::Px(22.),
        }
        TextColor(style_text_primary)
      ]
    });
  }

  if let Some(question) = panel_question {
    let size = select(18.0_f32, 22.0_f32, has_title);
    let color = select(style_text_secondary, style_text_primary, has_title);

    commands.spawn_scene(bsn! {
      ChildOf(panel_entity)
      QuestionTextRoot
      Text::new(question)
      TextFont {
        font_size: FontSize::Px(size),
      }
      TextColor({color})
    });
  } else {
    commands.spawn_scene(bsn! {
      ChildOf(panel_entity)
      Node {
        width: Val::Percent(100.0),
        height: Val::Auto,
        justify_content: JustifyContent::SpaceBetween,
        align_items: AlignItems::Center,
      }
      BackgroundColor(Color::NONE)
      Children [
      CategoryButton
      @button(style_surface, style_border)
      on(observe_category_dropdown_click)
      Node {
        width: Val::Percent(100.0),
        border: UiRect::all(Val::Px(3.0)),
        padding: UiRect::axes(Val::Px(10.0), Val::Px(6.0)),
        border_radius: BorderRadius::all(Val::Px(8.0)),
      }
      Children [
        @label("Category: ")
        TextColor(style_text_primary)
        --
        Text::default()
        Node::default()
        Children [
          CategoryButtonText
          TextSpan::new(FeedbackCategory::General.label().to_string())
          TextFont {
            font_size: FontSize::Px(16.),
          }
          TextColor(style_text_primary)
        ]
      ]
      ]
    });

    let category_list = commands
      .spawn_scene(bsn! {
        ChildOf(panel_entity)
        CategoryList
        Node {
          width: Val::Px(318.0),
          flex_direction: FlexDirection::Row,
          flex_wrap: FlexWrap::Wrap,
          justify_content: JustifyContent::SpaceBetween,
          row_gap: Val::Px(4.0),
          padding: UiRect::all(Val::Px(8.0)),
          border: UiRect::all(Val::Px(1.0)),
          display: Display::None,
          position_type: PositionType::Absolute,
          top: Val::Px(110.0),
          left: Val::Px(49.0),
          border_radius: BorderRadius::bottom(Val::Px(8.)),
        }
        BackgroundColor(style_background)
        BorderColor::all(style_border)
        ZIndex(10)
      })
      .id();

    for cat in FeedbackCategory::ALL {
      commands.spawn_scene(bsn! {
        ChildOf(category_list)
        CategoryItem({*cat})
        @button(style_surface, style_border)
        on(observe_category_item_click)
        Node {
          width: Val::Percent(48.0),
          border: UiRect::all(Val::Px(1.0)),
          padding: UiRect::axes(Val::Px(8.0), Val::Px(6.0)),
          justify_content: JustifyContent::Center,
          border_radius: BorderRadius::all(Val::Px(8.0)),
        }
        Children [
          Text::default()
          Node::default()
          Children [
            TextSpan::new(cat.label().to_string())
            TextFont {
              font_size: FontSize::Px(14.),
            }
            TextColor(style_text_primary)
          ]
        ]
      });
    }
  }

  commands.spawn_scene(bsn! {
    ChildOf(panel_entity)
    Node {
      width: Val::Percent(100.0),
      min_height: Val::Px(180.0),
      border_radius: BorderRadius::all(Val::Px(8.0)),
    }
    Children [
    MessageInputRoot
    @panel(style_surface, style_border)
    Node {
      width: Val::Percent(100.0),
      border: UiRect::all(Val::Px(2.0)),
      overflow: Overflow::scroll_y(),
      padding: UiRect::all(Val::Px(10.0)),
      border_radius: BorderRadius::all(Val::Px(8.0)),
    }
    Children [
    MessageInput
    Node {
      width: Val::Percent(100.0),
      height: Val::Percent(100.0),
    }
    Text::new("")
    TextFont {
      font_size: FontSize::Px(16.),
    }
    TextColor(style_text_primary)
    bevy::ui_widgets::TextInput
    EditableText {
      allow_newlines: true,
      max_characters: {Some(1000)},
    }
    TabIndex(0)
    EntityCursor::System(SystemCursorIcon::Text)
    TextCursorStyle::default()
    ]
    ]
  });

  if allow_screenshot {
    commands.spawn_scene(bsn! {
      ChildOf(panel_entity)
      Node {
        width: Val::Percent(100.0),
        justify_content: JustifyContent::SpaceBetween,
        align_items: AlignItems::Center,
      }
      BackgroundColor(Color::NONE)
      Children [
      ScreenshotToggle
      @button(style_surface, style_border)
      on(observe_screenshot_toggle_click)
      Node {
        border: UiRect::all(Val::Px(2.0)),
        padding: UiRect::axes(Val::Px(8.0), Val::Px(6.0)),
        border_radius: BorderRadius::all(Val::Px(8.0)),
      }
      Children [
        @label("Include screenshot: ")
        TextColor(style_text_secondary)
        --
        Text::default()
        Node::default()
        Children [
          ScreenshotToggleText
          TextSpan::new("No")
          TextFont {
            font_size: FontSize::Px(14.),
          }
          TextColor(style_secondary)
        ]
      ]
      ]
    });
  }

  commands.spawn_scene(bsn! {
    ChildOf(panel_entity)
    ErrorText
    Text::new("")
    TextFont {
      font_size: FontSize::Px(13.),
    }
    TextColor(style_error)
  });

  commands.spawn_scene(bsn! {
    ChildOf(panel_entity)
    Node {
      width: Val::Percent(100.0),
      justify_content: JustifyContent::SpaceAround,
      align_items: AlignItems::Center,
      column_gap: Val::Px(8.0),
      margin: UiRect::top(Val::Px(15.)),
    }
    BackgroundColor(Color::NONE)
    Children [
    CancelButton
    @button(style_surface, style_border)
    on(observe_cancel_click)
    Node {
      border: UiRect::all(Val::Px(2.0)),
      padding: UiRect::axes(Val::Px(14.0), Val::Px(10.0)),
      border_radius: BorderRadius::all(Val::Px(8.0)),
    }
    Children [
      @label("Cancel")
      TextColor(style_text_secondary)
    ]
    --
    SubmitButton
    Button
    Hovered::default()
    ButtonHoverStyle {
      background: style_primary_hover,
      border: {style_border.with_alpha(0.5)},
    }
    ButtonPressedStyle {
      background: {style_primary_hover.with_alpha(0.5)},
      border: {style_border.with_alpha(0.2)},
    }
    @panel(style_primary, style_primary_hover)
    on(observe_submit_click)
    Node {
      border: UiRect::all(Val::Px(2.0)),
      padding: UiRect::axes(Val::Px(14.0), Val::Px(10.0)),
      border_radius: BorderRadius::all(Val::Px(8.0)),
    }
    Children [
      @label("Submit Feedback")
      TextColor(style_text_primary)
    ]
    ]
  });
}

/// Replaces the submitted form with its thank-you message.
pub(crate) fn show_thank_you(
  mut commands: Commands,
  form: Res<FeedbackFormState>,
  props: Res<FeedbackPanelProps>,
  styles: Res<FeedbackPanelStyles>,
  mut panels: Query<(Entity, &mut Node), (With<FeedbackPanelBody>, Without<FeedbackThankYou>)>,
) {
  if form.thank_you_remaining.is_none() {
    return;
  }

  for (entity, mut node) in &mut panels {
    let thank_you_message = props.thank_you_message.clone();
    let success = styles.success;
    node.align_items = AlignItems::Center;
    node.justify_content = JustifyContent::Center;
    commands.entity(entity).despawn_children().insert(FeedbackThankYou);
    commands.spawn_scene(bsn! {
      ChildOf(entity)
      Text::new(thank_you_message)
      TextLayout::justify(Justify::Center)
      TextFont {
        font_size: FontSize::Px(22.0),
      }
      TextColor(success)
    });
  }
}

/// Syncs `FeedbackFormState::error` to the error text entity in the panel.
pub fn error_sync(form: Res<FeedbackFormState>, mut q: Query<&mut Text, With<ErrorText>>) {
  let Ok(mut text) = q.single_mut() else {
    return;
  };
  **text = form.error.clone().unwrap_or_default();
}

#[cfg(test)]
mod tests {
  use super::*;

  fn feedback_app(props: FeedbackPanelProps) -> App {
    let mut app = App::new();
    app
      .add_plugins((MinimalPlugins, AssetPlugin::default(), bevy::scene::ScenePlugin))
      .init_asset::<Font>()
      .init_resource::<FeedbackPanelStyles>()
      .insert_resource(props)
      .init_resource::<FeedbackFormState>()
      .add_systems(Startup, spawn_feedback_ui);
    app.update();
    app
  }

  #[test]
  fn message_field_is_registered_as_multiline_text_input() {
    let mut app = feedback_app(FeedbackPanelProps::default());

    let mut query = app
      .world_mut()
      .query_filtered::<(&EditableText, Has<bevy::ui_widgets::TextInput>), With<MessageInput>>();
    let (editable, is_text_input) = query.single(app.world()).unwrap();
    assert!(is_text_input);
    assert!(editable.allow_newlines);
    assert_eq!(editable.max_characters, Some(1000));
  }

  #[test]
  fn default_panel_spawns_categories_and_styled_controls() {
    let mut app = feedback_app(FeedbackPanelProps::default());
    let root = app
      .world_mut()
      .query_filtered::<Entity, With<FeedbackPanel>>()
      .single(app.world())
      .unwrap();
    let mut body_query = app
      .world_mut()
      .query_filtered::<(Entity, &ChildOf), With<FeedbackPanelBody>>();
    let (body, parent) = body_query.single(app.world()).unwrap();
    assert_eq!(parent.parent(), root);

    let mut list_query = app
      .world_mut()
      .query_filtered::<(Entity, &Node, &ChildOf), With<CategoryList>>();
    let (list, node, parent) = list_query.single(app.world()).unwrap();
    assert_eq!(parent.parent(), body);
    assert_eq!(node.display, Display::None);
    let mut items = app.world_mut().query::<(&CategoryItem, &ChildOf)>();
    let categories: Vec<_> = items
      .iter(app.world())
      .map(|(item, parent)| {
        assert_eq!(parent.parent(), list);
        item.0
      })
      .collect();
    assert_eq!(categories, FeedbackCategory::ALL);

    let mut buttons = app
      .world_mut()
      .query_filtered::<(&Hovered, &ButtonHoverStyle, &ButtonPressedStyle), With<Button>>();
    assert_eq!(buttons.iter(app.world()).count(), FeedbackCategory::ALL.len() + 4);
    let mut toggle = app
      .world_mut()
      .query_filtered::<Has<HoldPressed>, With<ScreenshotToggle>>();
    assert!(toggle.single(app.world()).unwrap());
  }

  #[test]
  fn question_panel_omits_category_and_screenshot_controls() {
    let mut app = feedback_app(FeedbackPanelProps::with_question("How was the level?", FeedbackCategory::Gameplay));
    let mut question = app.world_mut().query_filtered::<&Text, With<QuestionTextRoot>>();
    assert_eq!(question.single(app.world()).unwrap().0, "How was the level?");
    let mut optional_controls = app
      .world_mut()
      .query_filtered::<Entity, Or<(With<CategoryButton>, With<CategoryList>, With<ScreenshotToggle>)>>();
    assert_eq!(optional_controls.iter(app.world()).count(), 0);
    let form = app.world().resource::<FeedbackFormState>();
    assert_eq!(form.category, FeedbackCategory::Gameplay);
    assert_eq!(form.question.as_deref(), Some("How was the level?"));
  }

  #[test]
  fn respawning_panel_removes_old_hierarchy_and_resets_form() {
    let mut app = feedback_app(FeedbackPanelProps::default());
    let mut nodes = app.world_mut().query_filtered::<Entity, With<Node>>();
    let old_nodes: Vec<_> = nodes.iter(app.world()).collect();
    app.world_mut().resource_mut::<FeedbackFormState>().message = "Unsent feedback".to_string();
    app.add_systems(Update, spawn_feedback_ui);
    app.update();

    assert!(old_nodes.iter().all(|entity| app.world().get_entity(*entity).is_err()));
    let mut roots = app.world_mut().query_filtered::<Entity, With<FeedbackPanel>>();
    assert_eq!(roots.iter(app.world()).count(), 1);
    assert!(app.world().resource::<FeedbackFormState>().message.is_empty());
  }

  #[test]
  fn thank_you_replaces_form_once_and_preserves_panel_body() {
    let mut app = feedback_app(FeedbackPanelProps::default().thank_you_message("Feedback received"));
    let body = app
      .world_mut()
      .query_filtered::<Entity, With<FeedbackPanelBody>>()
      .single(app.world())
      .unwrap();
    app.world_mut().resource_mut::<FeedbackFormState>().thank_you_remaining = Some(std::time::Duration::from_secs(2));
    app.add_systems(Update, show_thank_you);
    app.update();

    assert!(app.world().get::<FeedbackThankYou>(body).is_some());
    let children = app.world().get::<Children>(body).unwrap();
    assert_eq!(children.len(), 1);
    let text_entity = children[0];
    assert_eq!(app.world().get::<Text>(text_entity).unwrap().0, "Feedback received");
    assert_eq!(
      app.world().get::<TextColor>(text_entity).unwrap().0,
      app.world().resource::<FeedbackPanelStyles>().success
    );
    let mut inputs = app.world_mut().query_filtered::<Entity, With<MessageInput>>();
    assert_eq!(inputs.iter(app.world()).count(), 0);

    app.update();
    assert_eq!(app.world().get::<Children>(body).unwrap()[0], text_entity);
  }
}
