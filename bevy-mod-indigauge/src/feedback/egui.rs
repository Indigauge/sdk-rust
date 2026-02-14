use bevy::prelude::*;
use bevy_egui::{EguiContexts, EguiPlugin, egui};

use crate::{
  feedback::{
    components::FeedbackPanel,
    observers::submit_feedback,
    resources::{FeedbackFormState, FeedbackKeyCodeToggle, FeedbackPanelProps, FeedbackPanelStyles},
    types::FeedbackSpawnPosition,
  },
  session::resources::SessionApiKey,
  utils::BevyIndigauge,
};

pub fn spawn_feedback_marker(mut commands: Commands, query: Query<Entity, With<FeedbackPanel>>) {
  if query.is_empty() {
    commands.spawn(FeedbackPanel);
  }
}

pub fn despawn_feedback_marker(mut commands: Commands, query: Query<Entity, With<FeedbackPanel>>) {
  for entity in &query {
    commands.entity(entity).despawn_recursive();
  }
}

pub fn toggle_panel_visibility_with_key(
  mut commands: Commands,
  keys: Res<ButtonInput<KeyCode>>,
  toggle_button: Res<FeedbackKeyCodeToggle>,
) {
  if keys.just_pressed(toggle_button.0) {
    commands.insert_resource(FeedbackPanelProps::default());
  }
}

pub fn draw_feedback_ui(
  mut commands: Commands,
  mut egui_contexts: EguiContexts,
  styles: Res<FeedbackPanelStyles>,
  props: Res<FeedbackPanelProps>,
  mut form: ResMut<FeedbackFormState>,
  mut ig: BevyIndigauge,
  session_key: Res<SessionApiKey>,
) {
  if !props.visible {
    return;
  }

  if props.is_changed() {
    *form = FeedbackFormState::default();

    if let Some(category) = props.category {
      form.category = category;
    }

    if let Some(question) = &props.question {
      form.question = Some(question.clone());
    }
  }

  let ctx = egui_contexts.ctx_mut();
  let panel_id = egui::Id::new("indigauge_feedback_panel");
  let frame = egui::Frame::none()
    .fill(to_egui_color(styles.background))
    .stroke(egui::Stroke::new(2.0, to_egui_color(styles.border)))
    .rounding(egui::Rounding::same(8.0))
    .inner_margin(egui::Margin::symmetric(48.0, 32.0));

  let margin = panel_margin(&props.position_margin);

  egui::Area::new(panel_id)
    .order(egui::Order::Foreground)
    .anchor(area_anchor(&props.spawn_position), panel_offset(&props.spawn_position, margin))
    .show(ctx, |ui| {
      frame.show(ui, |ui| {
        // ui.set_min_width(420.0);
        ui.set_max_width(420.0);
        // ui.set_min_height(420.0);

        if let Some(title) = &props.title {
          ui.colored_label(to_egui_color(styles.text_primary), egui::RichText::new(title).size(22.0));
          ui.add_space(10.0);
        }

        if let Some(question) = &props.question {
          let size = if props.title.is_some() { 18.0 } else { 22.0 };
          let color = if props.title.is_some() {
            styles.text_secondary
          } else {
            styles.text_primary
          };
          ui.colored_label(to_egui_color(color), egui::RichText::new(question).size(size));
          ui.add_space(10.0);
        } else {
          // Category label + dropdown (ComboBox) styled to match panel surface/border
          ui.horizontal(|ui| {
            ui.colored_label(to_egui_color(styles.text_secondary), egui::RichText::new("Category:").size(16.0));

            ui.scope(|ui| {
              let visuals = &mut ui.style_mut().visuals;
              visuals.widgets.inactive.bg_fill = to_egui_color(styles.surface);
              visuals.widgets.inactive.bg_stroke = egui::Stroke::new(2.0, to_egui_color(styles.border));
              visuals.widgets.inactive.fg_stroke.color = to_egui_color(styles.text_primary);

              visuals.widgets.hovered.bg_fill = to_egui_color(styles.surface.with_alpha(0.5));
              visuals.widgets.hovered.bg_stroke = egui::Stroke::new(2.0, to_egui_color(styles.border.with_alpha(0.5)));
              visuals.widgets.hovered.fg_stroke.color = to_egui_color(styles.text_primary);

              visuals.widgets.active.bg_fill = to_egui_color(styles.surface.with_alpha(0.5));
              visuals.widgets.active.bg_stroke = egui::Stroke::new(2.0, to_egui_color(styles.border.with_alpha(0.2)));
              visuals.widgets.active.fg_stroke.color = to_egui_color(styles.text_primary);

              egui::Frame::none()
                .fill(to_egui_color(styles.surface))
                .stroke(egui::Stroke::new(2.0, to_egui_color(styles.border)))
                .rounding(egui::Rounding::same(10.0))
                .inner_margin(egui::Margin::same(8.0))
                .show(ui, |ui| {
                  ui.set_min_width(260.0);
                  egui::ComboBox::from_label("")
                    .selected_text(
                      egui::RichText::new(form.category.label())
                        .size(16.0)
                        .color(to_egui_color(readable_text_color(styles.surface))),
                    )
                    .show_ui(ui, |ui| {
                      for category in crate::feedback::types::FeedbackCategory::ALL.iter() {
                        ui.selectable_value(&mut form.category, *category, category.label());
                      }
                    });
                });
            });
          });

          ui.add_space(10.0);
        }

        egui::Frame::none()
          .fill(to_egui_color(styles.surface))
          .stroke(egui::Stroke::new(2.0, to_egui_color(styles.border)))
          .rounding(egui::Rounding::same(8.0))
          .inner_margin(egui::Margin::same(10.0))
          .show(ui, |ui| {
            ui.add(
              egui::TextEdit::multiline(&mut form.message)
                .desired_width(f32::INFINITY)
                .desired_rows(8)
                .hint_text("Provide feedback message here"),
            );
          });

        if props.allow_screenshot {
          ui.add_space(10.0);
          let screenshot_text = if form.include_screenshot { "Yes" } else { "No" };
          let screenshot_color = if form.include_screenshot {
            styles.accent
          } else {
            styles.surface
          };

          if styled_button(
            ui,
            format!("Include screenshot: {screenshot_text}"),
            screenshot_color,
            styles.border,
            styles.text_secondary,
            styles.secondary_hover,
          )
          .clicked()
          {
            form.include_screenshot = !form.include_screenshot;
          }
        }

        if let Some(error) = &form.error {
          ui.add_space(8.0);
          ui.colored_label(to_egui_color(styles.error), error);
        }

        ui.add_space(15.0);
        ui.horizontal_centered(|ui| {
          if styled_button(
            ui,
            "Cancel",
            styles.surface,
            styles.border,
            styles.text_secondary,
            styles.surface.with_alpha(0.5),
          )
          .clicked()
          {
            commands.remove_resource::<FeedbackPanelProps>();
          }

          if styled_button(
            ui,
            "Submit Feedback",
            styles.primary,
            styles.border,
            styles.text_primary,
            styles.primary_hover,
          )
          .clicked()
          {
            let message = form.message.clone();
            submit_feedback(&mut commands, &mut form, &mut ig, &session_key, message);
          }
        });
      });
    });
}

fn styled_button(
  ui: &mut egui::Ui,
  label: impl Into<String>,
  background: Color,
  border: Color,
  text: Color,
  hover_background: Color,
) -> egui::Response {
  ui.scope(|ui| {
    let visuals = &mut ui.style_mut().visuals;
    visuals.widgets.inactive.bg_fill = to_egui_color(background);
    visuals.widgets.inactive.bg_stroke = egui::Stroke::new(2.0, to_egui_color(border));
    visuals.widgets.inactive.fg_stroke.color = to_egui_color(text);

    visuals.widgets.hovered.bg_fill = to_egui_color(hover_background);
    visuals.widgets.hovered.bg_stroke = egui::Stroke::new(2.5, to_egui_color(border.with_alpha(0.6)));
    visuals.widgets.hovered.fg_stroke.color = to_egui_color(text);

    visuals.widgets.active.bg_fill = to_egui_color(hover_background.with_alpha(0.6));
    visuals.widgets.active.bg_stroke = egui::Stroke::new(2.0, to_egui_color(border.with_alpha(0.25)));

    let rt_text_color = readable_text_color(background);
    let rt = egui::RichText::new(label)
      .size(16.0)
      .color(to_egui_color(rt_text_color));
    ui.add(
      egui::Button::new(rt)
        .fill(to_egui_color(background))
        .stroke(egui::Stroke::new(2.0, to_egui_color(border)))
        .rounding(egui::Rounding::same(10.0))
        .min_size(egui::vec2(120.0, 38.0))
        .sense(egui::Sense::click()),
    )
  })
  .inner
}

fn to_egui_color(color: Color) -> egui::Color32 {
  let [r, g, b, a] = color.to_srgba().to_u8_array();
  egui::Color32::from_rgba_unmultiplied(r, g, b, a)
}

/// Return a high-contrast foreground color (black or white) for `bg`.
///
/// Uses standard relative luminance (sRGB -> linear) to decide whether the
/// background is light or dark. If `bg` is light, returns `Color::BLACK`,
/// otherwise returns `Color::WHITE`.
pub(crate) fn readable_text_color(bg: Color) -> Color {
  // Convert to sRGBA floats in 0.0..1.0
  let [r8, g8, b8, _a] = bg.to_srgba().to_u8_array();
  let rs = r8 as f32 / 255.0;
  let gs = g8 as f32 / 255.0;
  let bs = b8 as f32 / 255.0;

  // sRGB -> linear conversion
  fn srgb_to_linear(c: f32) -> f32 {
    if c <= 0.04045 {
      c / 12.92
    } else {
      ((c + 0.055) / 1.055).powf(2.4)
    }
  }

  let r_lin = srgb_to_linear(rs);
  let g_lin = srgb_to_linear(gs);
  let b_lin = srgb_to_linear(bs);

  // Relative luminance (Rec. 709 / ITU-R BT.709)
  let lum = 0.2126 * r_lin + 0.7152 * g_lin + 0.0722 * b_lin;

  if lum > 0.5 { Color::BLACK } else { Color::WHITE }
}

fn val_to_px(value: Val) -> f32 {
  match value {
    Val::Px(value) => value,
    _ => 0.0,
  }
}

fn panel_margin(margin: &UiRect) -> [f32; 4] {
  [
    val_to_px(margin.left),
    val_to_px(margin.right),
    val_to_px(margin.top),
    val_to_px(margin.bottom),
  ]
}

fn area_anchor(spawn_position: &FeedbackSpawnPosition) -> egui::Align2 {
  match spawn_position {
    FeedbackSpawnPosition::TopLeft => egui::Align2::LEFT_TOP,
    FeedbackSpawnPosition::TopRight => egui::Align2::RIGHT_TOP,
    FeedbackSpawnPosition::TopCenter => egui::Align2::CENTER_TOP,
    FeedbackSpawnPosition::BottomLeft => egui::Align2::LEFT_BOTTOM,
    FeedbackSpawnPosition::BottomRight => egui::Align2::RIGHT_BOTTOM,
    FeedbackSpawnPosition::BottomCenter => egui::Align2::CENTER_BOTTOM,
    FeedbackSpawnPosition::Center => egui::Align2::CENTER_CENTER,
    FeedbackSpawnPosition::CenterLeft => egui::Align2::LEFT_CENTER,
    FeedbackSpawnPosition::CenterRight => egui::Align2::RIGHT_CENTER,
  }
}

fn panel_offset(spawn_position: &FeedbackSpawnPosition, margin: [f32; 4]) -> egui::Vec2 {
  let x = match spawn_position {
    FeedbackSpawnPosition::TopLeft | FeedbackSpawnPosition::BottomLeft | FeedbackSpawnPosition::CenterLeft => margin[0],
    FeedbackSpawnPosition::TopRight | FeedbackSpawnPosition::BottomRight | FeedbackSpawnPosition::CenterRight => {
      -margin[1]
    },
    _ => 0.0,
  };

  let y = match spawn_position {
    FeedbackSpawnPosition::TopLeft | FeedbackSpawnPosition::TopRight | FeedbackSpawnPosition::TopCenter => margin[2],
    FeedbackSpawnPosition::BottomLeft | FeedbackSpawnPosition::BottomRight | FeedbackSpawnPosition::BottomCenter => {
      -margin[3]
    },
    _ => 0.0,
  };

  egui::vec2(x, y)
}

pub fn ensure_egui_plugin(app: &mut App) {
  if !app.is_plugin_added::<EguiPlugin>() {
    app.add_plugins(EguiPlugin);
  }
}
