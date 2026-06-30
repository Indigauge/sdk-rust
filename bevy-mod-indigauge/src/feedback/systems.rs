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
  picking::hover::HoverMap,
  prelude::*,
  text::{EditableText, TextCursorStyle},
  window::SystemCursorIcon,
};
use bevy_feathers::cursor::EntityCursor;
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
    &'a Interaction,
    Entity,
    &'a mut BackgroundColor,
    &'a mut BorderColor,
    Option<&'a ButtonHoverStyle>,
    Option<&'a ButtonPressedStyle>,
    Option<&'a OriginalButtonStyles>,
    Has<HoldPressed>,
    Has<Active>,
  ),
  (Changed<Interaction>, Or<(With<ButtonHoverStyle>, With<ButtonPressedStyle>)>),
>;

// Handle hover and click states
/// Applies hover/pressed style transitions to feedback UI buttons.
pub fn handle_hover_and_click_styles(mut commands: Commands, mut q: HoverAndClickInteractionQuery) {
  q.iter_mut().for_each(
    |(interaction, entity, mut bg_color, mut border_color, bhs, bps, obs, hold_after_press, is_active)| {
      match *interaction {
        Interaction::Hovered => {
          if let Ok(mut ecm) = commands.get_entity(entity) {
            ecm.try_insert_if_new(OriginalButtonStyles {
              background: bg_color.0,
              border: border_color.top,
            });
          }

          if !is_active && let Some(hover_style) = bhs {
            bg_color.0 = hover_style.background;
            *border_color = BorderColor::all(hover_style.border);
          }
        },
        Interaction::Pressed => {
          if let Some(pressed_style) = bps {
            bg_color.0 = pressed_style.background;
            *border_color = BorderColor::all(pressed_style.border);
          }

          if hold_after_press && let Ok(mut ecm) = commands.get_entity(entity) {
            ecm.try_insert(Active);
          }
        },
        _ => {
          if !is_active && let Some(original_styles) = obs {
            bg_color.0 = original_styles.background;
            *border_color = BorderColor::all(original_styles.border);
          }
        },
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
      Node {
        width: Val::Percent(100.0),
        height: Val::Percent(100.0),
        align_items: panel_align_items,
        justify_content: panel_justify_content,
      }
      BackgroundColor(Color::NONE)
    })
    .id();
  commands.entity(root_entity).insert(FeedbackPanel);

  let panel_entity = commands
    .spawn_scene(bsn! {
      ChildOf(root_entity)
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
  commands
    .entity(panel_entity)
    .insert(panel(style_background, style_border));

  if let Some(title) = panel_title {
    let title_root = commands
      .spawn_scene(bsn! {
        ChildOf(panel_entity)
        Text::default()
        Node::default()
      })
      .id();

    commands.spawn_scene(bsn! {
      ChildOf(title_root)
      TextSpan::new(title)
      TextFont {
        font_size: FontSize::Px(22.),
      }
      TextColor(style_text_primary)
    });
  }

  if let Some(question) = panel_question {
    let size = select(18., 22., has_title);
    let color = select(style_text_secondary, style_text_primary, has_title);

    let question_entity = commands
      .spawn_scene(bsn! {
        ChildOf(panel_entity)
        Text::new(question)
        TextFont {
          font_size: FontSize::Px(size),
        }
        TextColor({color})
      })
      .id();
    commands.entity(question_entity).insert(QuestionTextRoot);
  } else {
    let category_row = commands
      .spawn_scene(bsn! {
        ChildOf(panel_entity)
        Node {
          width: Val::Percent(100.0),
          height: Val::Auto,
          justify_content: JustifyContent::SpaceBetween,
          align_items: AlignItems::Center,
        }
        BackgroundColor(Color::NONE)
      })
      .id();

    let category_button = commands
      .spawn_scene(bsn! {
        ChildOf(category_row)
        Node {
          width: Val::Percent(100.0),
          border: UiRect::all(Val::Px(3.0)),
          padding: UiRect::axes(Val::Px(10.0), Val::Px(6.0)),
          border_radius: BorderRadius::all(Val::Px(8.0)),
        }
      })
      .id();
    commands
      .entity(category_button)
      .insert(CategoryButton)
      .insert(button(style_surface, style_border))
      .observe(observe_category_dropdown_click);

    let category_label = commands.spawn_scene(label("Category: ")).id();
    commands
      .entity(category_label)
      .insert(ChildOf(category_button))
      .insert(TextColor(style_text_primary));

    let category_text_root = commands
      .spawn_scene(bsn! {
        ChildOf(category_button)
        Text::default()
        Node::default()
      })
      .id();
    let category_text = commands
      .spawn_scene(bsn! {
        ChildOf(category_text_root)
        TextSpan::new(FeedbackCategory::General.label().to_string())
        TextFont {
          font_size: FontSize::Px(16.),
        }
        TextColor(style_text_primary)
      })
      .id();
    commands.entity(category_text).insert(CategoryButtonText);

    let category_list = commands
      .spawn_scene(bsn! {
        ChildOf(panel_entity)
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
    commands.entity(category_list).insert(CategoryList);

    for cat in FeedbackCategory::ALL {
      let item = commands
        .spawn_scene(bsn! {
          ChildOf(category_list)
          Node {
            width: Val::Percent(48.0),
            border: UiRect::all(Val::Px(1.0)),
            padding: UiRect::axes(Val::Px(8.0), Val::Px(6.0)),
            justify_content: JustifyContent::Center,
            border_radius: BorderRadius::all(Val::Px(8.0)),
          }
        })
        .id();
      commands
        .entity(item)
        .insert(CategoryItem(*cat))
        .insert(button(style_surface, style_border))
        .observe(observe_category_item_click);

      let item_text_root = commands
        .spawn_scene(bsn! {
          ChildOf(item)
          Text::default()
          Node::default()
        })
        .id();

      commands.spawn_scene(bsn! {
        ChildOf(item_text_root)
        TextSpan::new(cat.label().to_string())
        TextFont {
          font_size: FontSize::Px(14.),
        }
        TextColor(style_text_primary)
      });
    }
  }

  let message_area = commands
    .spawn_scene(bsn! {
      ChildOf(panel_entity)
      Node {
        width: Val::Percent(100.0),
        min_height: Val::Px(180.0),
        border_radius: BorderRadius::all(Val::Px(8.0)),
      }
    })
    .id();

  let message_root = commands
    .spawn_scene(bsn! {
      ChildOf(message_area)
      Node {
        width: Val::Percent(100.0),
        border: UiRect::all(Val::Px(2.0)),
        overflow: Overflow::scroll_y(),
        padding: UiRect::all(Val::Px(10.0)),
        border_radius: BorderRadius::all(Val::Px(8.0)),
      }
    })
    .id();
  commands
    .entity(message_root)
    .insert(MessageInputRoot)
    .insert(panel(style_surface, style_border));

  let message_input = commands
    .spawn((
      ChildOf(message_root),
      Node {
        width: Val::Percent(100.0),
        height: Val::Percent(100.0),
        ..default()
      },
      Text::new(""),
      TextFont::from_font_size(FontSize::Px(16.)),
      TextColor(style_text_primary),
      EditableText {
        allow_newlines: true,
        max_characters: Some(1000),
        ..Default::default()
      },
      TabIndex(0),
      EntityCursor::System(SystemCursorIcon::Text),
      TextCursorStyle::default(),
    ))
    .id();
  commands.entity(message_input).insert(MessageInput);

  if allow_screenshot {
    let screenshot_row = commands
      .spawn_scene(bsn! {
        ChildOf(panel_entity)
        Node {
          width: Val::Percent(100.0),
          justify_content: JustifyContent::SpaceBetween,
          align_items: AlignItems::Center,
        }
        BackgroundColor(Color::NONE)
      })
      .id();

    let screenshot_toggle = commands
      .spawn_scene(bsn! {
        ChildOf(screenshot_row)
        Node {
          border: UiRect::all(Val::Px(2.0)),
          padding: UiRect::axes(Val::Px(8.0), Val::Px(6.0)),
          border_radius: BorderRadius::all(Val::Px(8.0)),
        }
      })
      .id();
    commands
      .entity(screenshot_toggle)
      .insert(ScreenshotToggle)
      .insert(button(style_surface, style_border))
      .observe(observe_screenshot_toggle_click);

    let screenshot_label = commands.spawn_scene(label("Include screenshot: ")).id();
    commands
      .entity(screenshot_label)
      .insert(ChildOf(screenshot_toggle))
      .insert(TextColor(style_text_secondary));

    let screenshot_text_root = commands
      .spawn_scene(bsn! {
        ChildOf(screenshot_toggle)
        Text::default()
        Node::default()
      })
      .id();

    let screenshot_text = commands
      .spawn_scene(bsn! {
        ChildOf(screenshot_text_root)
        TextSpan::new("No")
        TextFont {
          font_size: FontSize::Px(14.),
        }
        TextColor(style_secondary)
      })
      .id();
    commands.entity(screenshot_text).insert(ScreenshotToggleText);
  }

  let error_entity = commands
    .spawn_scene(bsn! {
      ChildOf(panel_entity)
      Text::new("")
      TextFont {
        font_size: FontSize::Px(13.),
      }
      TextColor(style_error)
    })
    .id();
  commands.entity(error_entity).insert(ErrorText);

  let actions_row = commands
    .spawn_scene(bsn! {
      ChildOf(panel_entity)
      Node {
        width: Val::Percent(100.0),
        justify_content: JustifyContent::SpaceAround,
        align_items: AlignItems::Center,
        column_gap: Val::Px(8.0),
        margin: UiRect::top(Val::Px(15.)),
      }
      BackgroundColor(Color::NONE)
    })
    .id();

  let cancel_button = commands
    .spawn_scene(bsn! {
      ChildOf(actions_row)
      Node {
        border: UiRect::all(Val::Px(2.0)),
        padding: UiRect::axes(Val::Px(14.0), Val::Px(10.0)),
        border_radius: BorderRadius::all(Val::Px(8.0)),
      }
    })
    .id();
  commands
    .entity(cancel_button)
    .insert(CancelButton)
    .insert(button(style_surface, style_border))
    .observe(observe_cancel_click);
  let cancel_label = commands.spawn_scene(label("Cancel")).id();
  commands
    .entity(cancel_label)
    .insert(ChildOf(cancel_button))
    .insert(TextColor(style_text_secondary));

  let submit_button = commands
    .spawn_scene(bsn! {
      ChildOf(actions_row)
      Node {
        border: UiRect::all(Val::Px(2.0)),
        padding: UiRect::axes(Val::Px(14.0), Val::Px(10.0)),
        border_radius: BorderRadius::all(Val::Px(8.0)),
      }
    })
    .id();
  commands
    .entity(submit_button)
    .insert(Button)
    .insert(SubmitButton)
    .insert(ButtonHoverStyle {
      background: style_primary_hover,
      border: style_border.with_alpha(0.5),
    })
    .insert(ButtonPressedStyle {
      background: style_primary_hover.with_alpha(0.5),
      border: style_border.with_alpha(0.2),
    })
    .insert(panel(style_primary, style_primary_hover))
    .observe(observe_submit_click);
  let submit_label = commands.spawn_scene(label("Submit Feedback")).id();
  commands
    .entity(submit_label)
    .insert(ChildOf(submit_button))
    .insert(TextColor(style_text_primary));
}

/// Syncs `FeedbackFormState::error` to the error text entity in the panel.
pub fn error_sync(form: Res<FeedbackFormState>, mut q: Query<&mut Text, With<ErrorText>>) {
  let Ok(mut text) = q.single_mut() else {
    return;
  };
  **text = form.error.clone().unwrap_or_default();
}
