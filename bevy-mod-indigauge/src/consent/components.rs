use bevy::prelude::*;

/// Marker component for the active consent modal root node.
#[derive(Component)]
pub struct ConsentModal;

/// Marker for the accept-consent button.
#[derive(Component)]
pub struct AcceptConsentButton;

/// Marker for the decline-consent button.
#[derive(Component)]
pub struct DeclineConsentButton;
