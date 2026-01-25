//! Reader plugin - orchestrates the reading experience.
//!
//! Manages reading state transitions and timing. Delegates to specialized modules
//! for input handling, ORP display, and timing calculations.

use bevy::prelude::*;

use crate::input::InputPlugin;
use crate::orp::OrpPlugin;
use crate::state::{ActiveTab, ReadingState, TabWpm, WordsManager};
use crate::timing::ReadingTimer;
use std::time::Duration;

pub struct ReaderPlugin;

impl Plugin for ReaderPlugin {
    fn build(&self, app: &mut App) {
        app.init_state::<ReadingState>()
            .init_resource::<ActiveTab>()
            .init_resource::<ReadingTimer>()
            .add_plugins((OrpPlugin, InputPlugin))
            .add_systems(Update, tick_reader.run_if(in_state(ReadingState::Active)))
            .add_systems(OnEnter(ReadingState::Active), start_reading)
            ;
    }
}

fn start_reading(
    mut timer: ResMut<ReadingTimer>,
    active_tab: Res<ActiveTab>,
    tabs: Query<(&TabWpm, &WordsManager)>,
) {
    let Some(entity) = active_tab.entity else { return };
    let Ok((tab_wpm, words_mgr)) = tabs.get(entity) else { return };
    if !words_mgr.words.is_empty() {
        let word = &words_mgr.words[words_mgr.current_index];
        let delay = Duration::from_millis(word.display_duration_ms(tab_wpm.0));
        timer.timer = Timer::new(delay, TimerMode::Once);
    }
}

fn tick_reader(
    time: Res<Time>,
    mut timer: ResMut<ReadingTimer>,
    active_tab: Res<ActiveTab>,
    mut tabs: Query<(&TabWpm, &mut WordsManager)>,
    mut next_state: ResMut<NextState<ReadingState>>,
) {
    let Some(entity) = active_tab.entity else { return };
    let Ok((tab_wpm, mut words_mgr)) = tabs.get_mut(entity) else { return };
    
    timer.timer.tick(time.delta());
    
    if timer.timer.just_finished() {
        if words_mgr.current_index + 1 < words_mgr.words.len() {
            words_mgr.current_index += 1;
            let word = &words_mgr.words[words_mgr.current_index];
            let delay = Duration::from_millis(word.display_duration_ms(tab_wpm.0));
            timer.timer = Timer::new(delay, TimerMode::Once);
        } else {
            next_state.set(ReadingState::Idle);
        }
    }
}
