use std::time::Instant;

use crate::{
  feedback::components::FeedbackPanel,
  feedback::resources::{FeedbackFormState, TakeScreenshot},
  http_runtime::ReqwestResponseEvent,
  prelude::*,
  session::resources::SessionApiKey,
  utils::BevyIndigauge,
};
use bevy::log::error;
use bevy::{
  prelude::*,
  render::view::screenshot::{Screenshot, ScreenshotCaptured},
  state::state::FreelyMutableState,
};
use image::{ColorType, ImageEncoder, codecs::png::PngEncoder};
use indigauge_core::types::{FeedbackPayload, IdResponse};
use std::ops::Deref;

#[cfg(all(feature = "feedback", not(feature = "feedback_egui")))]
use crate::feedback::components::{CategoryButtonText, CategoryItem, MessageInput, ScreenshotToggleText};
#[cfg(all(feature = "feedback", not(feature = "feedback_egui")))]
use bevy::text::EditableText;
#[cfg(all(feature = "feedback", not(feature = "feedback_egui")))]
use indigauge_core::utils::select;

/// Returns an observer that advances a state when feedback panel spawns.
pub fn switch_state_on_feedback_spawn<S>(state: S) -> impl FnMut(On<Add<FeedbackPanel>>, ResMut<NextState<S>>)
where
  S: FreelyMutableState + Copy,
{
  move |_trigger, mut next_state| {
    next_state.set(state);
  }
}

/// Returns an observer that advances a state when feedback panel despawns.
pub fn switch_state_on_feedback_despawn<S>(state: S) -> impl FnMut(On<Remove<FeedbackPanel>>, ResMut<NextState<S>>)
where
  S: FreelyMutableState + Copy,
{
  move |_trigger, mut next_state| {
    next_state.set(state);
  }
}

#[cfg(all(feature = "feedback", not(feature = "feedback_egui")))]
/// Toggles category dropdown open/closed state.
pub fn observe_category_dropdown_click(
  _trigger: On<PointerClick>,
  mut ui_state: ResMut<crate::feedback::resources::FeedbackUiState>,
) {
  ui_state.dropdown_open = !ui_state.dropdown_open;
}

#[cfg(all(feature = "feedback", not(feature = "feedback_egui")))]
/// Updates selected category from dropdown item interactions.
pub fn observe_category_item_click(
  trigger: On<PointerClick>,
  mut form: ResMut<FeedbackFormState>,
  category_item_query: Query<&CategoryItem>,
  mut q_btn_text_root: Query<&mut TextSpan, With<CategoryButtonText>>,
  mut ui_state: ResMut<crate::feedback::resources::FeedbackUiState>,
) {
  let Ok(CategoryItem(category)) = category_item_query.get(trigger.event().entity) else {
    return;
  };

  form.category = *category;
  ui_state.dropdown_open = false;

  // Update button text
  if let Ok(mut root) = q_btn_text_root.single_mut() {
    **root = category.label().to_string();
  }
}

#[cfg(all(feature = "feedback", not(feature = "feedback_egui")))]
/// Observer toggling screenshot inclusion for feedback submissions.
pub fn observe_screenshot_toggle_click(
  trigger: On<PointerClick>,
  styles: Res<FeedbackPanelStyles>,
  mut form: ResMut<FeedbackFormState>,
  mut q: Query<&mut BackgroundColor>,
  mut q_text_root: Query<(&mut TextSpan, &mut TextColor), With<ScreenshotToggleText>>,
) {
  let Ok(mut bg_color) = q.get_mut(trigger.event().entity) else {
    return;
  };

  form.include_screenshot = !form.include_screenshot;

  bg_color.0 = select(styles.accent, styles.surface, form.include_screenshot);

  if let Ok((mut root, mut color)) = q_text_root.single_mut() {
    **root = select("Yes", "No", form.include_screenshot).to_string();
    color.0 = select(styles.text_primary, styles.text_secondary, form.include_screenshot);
  }
}

#[cfg(all(feature = "feedback", not(feature = "feedback_egui")))]
/// Observer that closes the feedback panel.
pub fn observe_cancel_click(_trigger: On<PointerClick>, mut commands: Commands) {
  commands.remove_resource::<FeedbackPanelProps>();
}

#[cfg(all(feature = "feedback", not(feature = "feedback_egui")))]
/// Observer that reads form values and submits feedback.
pub fn observe_submit_click(
  _trigger: On<PointerClick>,
  mut commands: Commands,
  q_input: Query<&EditableText, With<MessageInput>>,
  mut form: ResMut<FeedbackFormState>,
  mut ig: BevyIndigauge,
  session_key: Res<SessionApiKey>,
  props: Res<FeedbackPanelProps>,
) {
  form.message = q_input.single().map(|s| s.value().to_string()).unwrap_or_default();
  let message = form.message.clone();
  submit_feedback(&mut commands, &mut form, &mut ig, &session_key, &props, message);
}

pub(crate) fn submit_feedback(
  commands: &mut Commands,
  form: &mut FeedbackFormState,
  ig: &mut BevyIndigauge,
  session_key: &SessionApiKey,
  props: &FeedbackPanelProps,
  message: String,
) {
  if form.pending_submission.is_some() || form.thank_you_remaining.is_some() {
    return;
  }

  if let Some(start_instant) = indigauge_core::state::get_session_start_instant() {
    let elapsed_ms = Instant::now().duration_since(*start_instant).as_millis();

    let msg = message
      .replace("\r\n", "\n")
      .replace('\r', "\n")
      .replace("  ", " ")
      .trim()
      .to_string();

    if msg.len().lt(&2) {
      form.error = Some("Feedback cannot be less than 2 characters".to_string());
      return;
    }

    if form.include_screenshot {
      commands.insert_resource(TakeScreenshot);
    } else {
      commands.remove_resource::<TakeScreenshot>();
    }

    let payload = FeedbackPayload {
      message: &msg,
      category: form.category.label().to_lowercase(),
      elapsed_ms,
      question: form.question.as_ref(),
    };
    form.error = None;
    match ig.send_feedback(session_key, &payload, maybe_take_screenshot) {
      Ok(Some(entity)) => form.pending_submission = Some(entity),
      Ok(None) => {
        commands.remove_resource::<TakeScreenshot>();
        form.complete_submission(props);
        if !props.show_thank_you {
          commands.remove_resource::<FeedbackPanelProps>();
        }
      },
      Err(error) => {
        commands.remove_resource::<TakeScreenshot>();
        form.error = Some(error);
      },
    }
  } else {
    form.error = Some("Failed to submit feedback: session start time not found".to_string());
  }
}

fn maybe_take_screenshot(
  trigger: On<ReqwestResponseEvent>,
  mut commands: Commands,
  take_screenshot: Option<Res<TakeScreenshot>>,
  props: Option<Res<FeedbackPanelProps>>,
  mut form: ResMut<FeedbackFormState>,
) {
  if form.pending_submission != Some(trigger.entity) {
    return;
  }
  commands.remove_resource::<TakeScreenshot>();
  if !trigger.status().is_success() {
    form.pending_submission = None;
    form.error = Some("Failed to send feedback. Please try again.".to_string());
    return;
  }
  if let Some(props) = props {
    form.complete_submission(&props);
    if !props.show_thank_you {
      commands.remove_resource::<FeedbackPanelProps>();
    }
  }
  if take_screenshot.is_some()
    && let Ok(feedback_id) = trigger.event().deserialize_json::<IdResponse>()
  {
    commands.remove_resource::<TakeScreenshot>();
    commands.spawn(Screenshot::primary_window()).observe(
      move |trigger: On<ScreenshotCaptured>, mut ig: BevyIndigauge, api_key: Res<SessionApiKey>| {
        let img = trigger.event().deref().clone();

        match img.try_into_dynamic() {
          Ok(dyn_img) => {
            let data = dyn_img.to_rgb8().to_vec();
            let mut png = Vec::new();
            let enc = PngEncoder::new(&mut png);

            if enc
              .write_image(&data, dyn_img.width(), dyn_img.height(), ColorType::Rgb8.into())
              .is_ok()
            {
              ig.send_feedback_screenshot(&api_key, &feedback_id, png);
            } else if **ig.log_level <= IndigaugeLogLevel::Error {
              error!(message = "Failed to encode screenshot as PNG");
            }
          },
          Err(error) => {
            if **ig.log_level <= IndigaugeLogLevel::Error {
              error!(message = "Failed to convert screenshot into dynamic image", ?error);
            }
          },
        }
      },
    );
  }
}

#[cfg(test)]
mod tests {
  use super::*;
  use crate::http_runtime::{HeaderMap, StatusCode};

  fn response_app(props: FeedbackPanelProps) -> (App, Entity) {
    let mut app = App::new();
    app.insert_resource(props).init_resource::<FeedbackFormState>();
    let entity = app.world_mut().spawn_empty().observe(maybe_take_screenshot).id();
    app.world_mut().resource_mut::<FeedbackFormState>().pending_submission = Some(entity);
    (app, entity)
  }

  fn respond(app: &mut App, entity: Entity, status: StatusCode) {
    app
      .world_mut()
      .trigger(ReqwestResponseEvent::new(entity, bytes::Bytes::new(), status, HeaderMap::new()));
    app.world_mut().flush();
  }

  #[test]
  fn success_keeps_panel_open_with_thank_you() {
    let (mut app, entity) = response_app(FeedbackPanelProps::default());
    respond(&mut app, entity, StatusCode::CREATED);
    let form = app.world().resource::<FeedbackFormState>();
    assert_eq!(form.thank_you_remaining, Some(std::time::Duration::from_secs(2)));
    assert!(form.pending_submission.is_none());
    assert!(app.world().contains_resource::<FeedbackPanelProps>());
  }

  #[test]
  fn disabled_confirmation_closes_after_success() {
    let (mut app, entity) = response_app(FeedbackPanelProps::default().show_thank_you(false));
    respond(&mut app, entity, StatusCode::OK);
    assert!(!app.world().contains_resource::<FeedbackPanelProps>());
  }

  #[test]
  fn failure_keeps_form_for_retry_without_thank_you() {
    let (mut app, entity) = response_app(FeedbackPanelProps::default());
    respond(&mut app, entity, StatusCode::INTERNAL_SERVER_ERROR);
    let form = app.world().resource::<FeedbackFormState>();
    assert!(form.thank_you_remaining.is_none());
    assert!(form.pending_submission.is_none());
    assert!(form.error.is_some());
    assert!(app.world().contains_resource::<FeedbackPanelProps>());
  }

  #[test]
  fn stale_success_does_not_change_replacement_form() {
    let (mut app, entity) = response_app(FeedbackPanelProps::default());
    app.world_mut().insert_resource(FeedbackFormState::default());
    respond(&mut app, entity, StatusCode::OK);
    assert!(
      app
        .world()
        .resource::<FeedbackFormState>()
        .thank_you_remaining
        .is_none()
    );
    assert!(app.world().contains_resource::<FeedbackPanelProps>());
  }
}
