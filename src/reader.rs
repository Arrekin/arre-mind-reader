//! Reader plugin - orchestrates the reading experience.
//!
//! Manages reading state transitions and timing. Delegates to specialized modules
//! for input handling, ORP display, and timing calculations.

use std::time::Duration;
use bevy::prelude::*;

use crate::input::InputPlugin;
use crate::orp::OrpPlugin;
use crate::state::{ActiveTab};
use crate::text::Word;
use crate::timing::ReadingTimer;

pub struct ReaderPlugin;
impl Plugin for ReaderPlugin {
    fn build(&self, app: &mut App) {
        app.init_state::<ReadingState>()
            .init_resource::<ActiveTab>()
            .init_resource::<ReadingTimer>()
            .add_plugins((OrpPlugin, InputPlugin))
            .add_systems(Update, ReadingState::tick.run_if(in_state(ReadingState::Playing)))
            .add_systems(OnEnter(ReadingState::Playing), ReadingState::on_start_playing)
            ;
    }
}


#[derive(States, Debug, Clone, PartialEq, Eq, Hash, Default)]
pub enum ReadingState {
    #[default]
    Idle,
    Playing,
    Paused,
}
impl ReadingState {
    fn on_start_playing(
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

    fn tick(
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
}

// ============================================================================
// Tab Components
// ============================================================================

pub type TabId = u64;

#[derive(Component)]
pub struct TabMarker {
    pub id: TabId,
}

#[derive(Component)]
pub struct TabFontSettings {
    pub font_name: String,
    pub font_handle: Handle<Font>,
    pub font_size: f32,
}

#[derive(Component)]
pub struct TabWpm(pub u32);

#[derive(Component)]
pub struct TabFilePath(pub std::path::PathBuf);

#[derive(Component)]
pub struct WordsManager {
    pub words: Vec<Word>,
    pub current_index: usize,
}
