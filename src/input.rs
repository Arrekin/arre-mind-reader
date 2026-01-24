//! Keyboard input handling for playback control.
//!
//! Handles play/pause, navigation, and WPM adjustment via keyboard shortcuts.

use bevy::prelude::*;

use crate::state::constants::*;
use crate::state::{ReaderSettings, ReaderState, ReadingState, TabManager};

pub fn handle_input(
    keyboard: Res<ButtonInput<KeyCode>>,
    current_state: Res<State<ReadingState>>,
    mut next_state: ResMut<NextState<ReadingState>>,
    mut reader_state: ResMut<ReaderState>,
    mut settings: ResMut<ReaderSettings>,
    tabs: Res<TabManager>,
) {
    // Space: toggle play/pause
    if keyboard.just_pressed(KeyCode::Space) {
        match current_state.get() {
            ReadingState::Idle | ReadingState::Paused => {
                next_state.set(ReadingState::Active);
            }
            ReadingState::Active => {
                next_state.set(ReadingState::Paused);
            }
        }
    }
    
    // Escape: stop
    if keyboard.just_pressed(KeyCode::Escape) {
        next_state.set(ReadingState::Idle);
    }
    
    // R: restart
    if keyboard.just_pressed(KeyCode::KeyR) {
        reader_state.current_index = 0;
    }
    
    let word_count = tabs.active_tab().map(|t| t.words.len()).unwrap_or(0);
    
    // Arrow keys: navigation and WPM
    if keyboard.just_pressed(KeyCode::ArrowLeft) {
        reader_state.current_index = reader_state.current_index.saturating_sub(WORD_SKIP_AMOUNT);
    }
    if keyboard.just_pressed(KeyCode::ArrowRight) {
        reader_state.current_index = (reader_state.current_index + WORD_SKIP_AMOUNT)
            .min(word_count.saturating_sub(1));
    }
    if keyboard.just_pressed(KeyCode::ArrowUp) {
        settings.wpm = (settings.wpm + WPM_STEP).min(WPM_MAX);
    }
    if keyboard.just_pressed(KeyCode::ArrowDown) {
        settings.wpm = settings.wpm.saturating_sub(WPM_STEP).max(WPM_MIN);
    }
}
