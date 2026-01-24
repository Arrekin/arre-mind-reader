//! Reader plugin - orchestrates the reading experience.
//!
//! Manages reading state transitions and timing. Delegates to specialized modules
//! for input handling, ORP display, and timing calculations.

use bevy::prelude::*;

use crate::input::handle_input;
use crate::orp::{setup_orp_display, update_word_display};
use crate::state::{ReaderSettings, ReaderState, ReadingState, TabManager};
use crate::timing::{calc_delay, ReadingTimer};

pub struct ReaderPlugin;

impl Plugin for ReaderPlugin {
    fn build(&self, app: &mut App) {
        app.init_state::<ReadingState>()
            .init_resource::<ReaderState>()
            .init_resource::<ReaderSettings>()
            .init_resource::<ReadingTimer>()
            .add_systems(Startup, setup_orp_display)
            .add_systems(Update, (
                handle_input,
                tick_reader.run_if(in_state(ReadingState::Active)),
                update_word_display,
            ))
            .add_systems(OnEnter(ReadingState::Active), start_reading);
    }
}

fn start_reading(
    mut timer: ResMut<ReadingTimer>,
    reader_state: Res<ReaderState>,
    tabs: Res<TabManager>,
    settings: Res<ReaderSettings>,
) {
    let Some(tab) = tabs.active_tab() else { return };
    if !tab.words.is_empty() {
        let word = &tab.words[reader_state.current_index];
        let delay = calc_delay(word, settings.wpm);
        timer.timer = Timer::new(delay, TimerMode::Once);
    }
}

fn tick_reader(
    time: Res<Time>,
    mut timer: ResMut<ReadingTimer>,
    mut reader_state: ResMut<ReaderState>,
    tabs: Res<TabManager>,
    settings: Res<ReaderSettings>,
    mut next_state: ResMut<NextState<ReadingState>>,
) {
    let Some(tab) = tabs.active_tab() else { return };
    
    timer.timer.tick(time.delta());
    
    if timer.timer.just_finished() {
        if reader_state.current_index + 1 < tab.words.len() {
            reader_state.current_index += 1;
            let word = &tab.words[reader_state.current_index];
            let delay = calc_delay(word, settings.wpm);
            timer.timer = Timer::new(delay, TimerMode::Once);
        } else {
            next_state.set(ReadingState::Idle);
        }
    }
}
