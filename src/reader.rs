//! Reader plugin - orchestrates the reading experience.
//!
//! Manages reading state transitions and timing.
//! for input handling, ORP display, and timing calculations.

use std::time::Duration;
use bevy::prelude::*;

use crate::input::InputPlugin;
use crate::orp::OrpPlugin;
use crate::text::Word;

pub const WPM_DEFAULT: u32 = 300;
pub const WPM_MIN: u32 = 100;
pub const WPM_MAX: u32 = 1000;
pub const WPM_STEP: u32 = 50;
pub const FONT_SIZE_DEFAULT: f32 = 48.0;

pub struct ReaderPlugin;
impl Plugin for ReaderPlugin {
    fn build(&self, app: &mut App) {
        app.init_state::<ReadingState>()
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
        active_tab: Single<(&TabWpm, &WordsManager), With<ActiveTab>>,
    ) {
        let (tab_wpm, words_mgr) = active_tab.into_inner();
        if !words_mgr.words.is_empty() {
            let word = &words_mgr.words[words_mgr.current_index];
            let delay = Duration::from_millis(word.display_duration_ms(tab_wpm.0));
            timer.timer = Timer::new(delay, TimerMode::Once);
        }
    }

    fn tick(
        time: Res<Time>,
        mut timer: ResMut<ReadingTimer>,
        active_tab: Single<(&TabWpm, &mut WordsManager), With<ActiveTab>>,
        mut next_state: ResMut<NextState<ReadingState>>,
    ) {
        let (tab_wpm, mut words_mgr) = active_tab.into_inner();
        
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

#[derive(Resource, Default)]
pub struct ReadingTimer {
    pub timer: Timer,
}


// ============================================================================
// Tab Components
// ============================================================================

#[derive(Component)]
#[component(storage = "SparseSet")]
pub struct ActiveTab;

#[derive(Component)]
pub struct TabMarker;

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
