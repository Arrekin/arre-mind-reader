//! Keyboard input handling for playback control.
//!
//! Handles play/pause, navigation, and WPM adjustment via keyboard shortcuts.

use bevy::prelude::*;

use crate::reader::{ActiveTab, ReadingState, TabWpm, WordsManager, WPM_MIN, WPM_MAX, WPM_STEP};

const WORD_SKIP_AMOUNT: usize = 5;

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
    current_state: Res<State<ReadingState>>,
    mut next_state: ResMut<NextState<ReadingState>>,
    mut active_tabs: Query<(&mut TabWpm, &mut WordsManager), With<ActiveTab>>,
) {
    // Space: toggle play/pause
    if keyboard.just_pressed(KeyCode::Space) {
        match current_state.get() {
            ReadingState::Idle | ReadingState::Paused => {
                next_state.set(ReadingState::Playing);
            }
            ReadingState::Playing => {
                next_state.set(ReadingState::Paused);
            }
        }
    }
    
    // Escape: stop
    if keyboard.just_pressed(KeyCode::Escape) {
        next_state.set(ReadingState::Idle);
    }
    
    let Ok((mut tab_wpm, mut words_mgr)) = active_tabs.single_mut() else { return };
    
    // R: restart
    if keyboard.just_pressed(KeyCode::KeyR) {
        words_mgr.current_index = 0;
    }
    
    let word_count = words_mgr.words.len();
    
    // Arrow keys: navigation and WPM
    if keyboard.just_pressed(KeyCode::ArrowLeft) {
        words_mgr.current_index = words_mgr.current_index.saturating_sub(WORD_SKIP_AMOUNT);
    }
    if keyboard.just_pressed(KeyCode::ArrowRight) {
        words_mgr.current_index = (words_mgr.current_index + WORD_SKIP_AMOUNT)
            .min(word_count.saturating_sub(1));
    }
    if keyboard.just_pressed(KeyCode::ArrowUp) {
        tab_wpm.0 = (tab_wpm.0 + WPM_STEP).min(WPM_MAX);
    }
    if keyboard.just_pressed(KeyCode::ArrowDown) {
        tab_wpm.0 = tab_wpm.0.saturating_sub(WPM_STEP).max(WPM_MIN);
    }
}
