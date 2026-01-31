//! Keyboard input handling for playback control.
//!
//! Handles play/pause, navigation, and WPM adjustment via keyboard shortcuts.

use bevy::prelude::*;

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
    keyboard: Res<ButtonInput<KeyCode>>,
    mut playback_cmds: MessageWriter<PlaybackCommand>,
) {
    // Space: toggle play/pause
    if keyboard.just_pressed(KeyCode::Space) {
        playback_cmds.write(PlaybackCommand::TogglePlayPause);
    }
    
    // Escape: stop
    if keyboard.just_pressed(KeyCode::Escape) {
        playback_cmds.write(PlaybackCommand::Stop);
    }
    
    // R: restart
    if keyboard.just_pressed(KeyCode::KeyR) {
        playback_cmds.write(PlaybackCommand::Restart);
    }
    
    // Arrow keys: navigation and WPM
    if keyboard.just_pressed(KeyCode::ArrowLeft) {
        playback_cmds.write(PlaybackCommand::skip_backward());
    }
    if keyboard.just_pressed(KeyCode::ArrowRight) {
        playback_cmds.write(PlaybackCommand::skip_forward());
    }
    if keyboard.just_pressed(KeyCode::ArrowUp) {
        playback_cmds.write(PlaybackCommand::AdjustWpm(WPM_STEP as i32));
    }
    if keyboard.just_pressed(KeyCode::ArrowDown) {
        playback_cmds.write(PlaybackCommand::AdjustWpm(-(WPM_STEP as i32)));
    }
}
