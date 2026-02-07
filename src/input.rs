//! Keyboard input handling for playback control.
//!
//! Handles play/pause, navigation, and WPM adjustment via keyboard shortcuts.

use bevy::prelude::*;
use bevy_egui::EguiContexts;

use crate::playback::PlaybackCommand;
use crate::reader::WPM_STEP;

pub struct InputPlugin;
impl Plugin for InputPlugin {
    fn build(&self, app: &mut App) {
        app
            .add_systems(Update, handle_input)
            ;
    }
}

fn handle_input(
    mut commands: Commands,
    keyboard: Res<ButtonInput<KeyCode>>,
    mut contexts: EguiContexts,
) {
    if contexts.ctx_mut().is_ok_and(|ctx| ctx.wants_keyboard_input()) {
        return;
    }

    // Space: toggle play/pause
    if keyboard.just_pressed(KeyCode::Space) {
        commands.trigger(PlaybackCommand::TogglePlayPause);
    }
    
    // Escape: stop
    if keyboard.just_pressed(KeyCode::Escape) {
        commands.trigger(PlaybackCommand::Stop);
    }
    
    // R: restart
    if keyboard.just_pressed(KeyCode::KeyR) {
        commands.trigger(PlaybackCommand::Restart);
    }
    
    // Arrow keys: navigation and WPM
    if keyboard.just_pressed(KeyCode::ArrowLeft) {
        commands.trigger(PlaybackCommand::skip_backward());
    }
    if keyboard.just_pressed(KeyCode::ArrowRight) {
        commands.trigger(PlaybackCommand::skip_forward());
    }
    if keyboard.just_pressed(KeyCode::ArrowUp) {
        commands.trigger(PlaybackCommand::AdjustWpm(WPM_STEP as i32));
    }
    if keyboard.just_pressed(KeyCode::ArrowDown) {
        commands.trigger(PlaybackCommand::AdjustWpm(-(WPM_STEP as i32)));
    }
}
