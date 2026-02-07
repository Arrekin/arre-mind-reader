//! Reader plugin - orchestrates the reading experience.
//!
//! Manages reading state transitions and timing.

use std::time::Duration;
use bevy::prelude::*;

use crate::input::InputPlugin;
use crate::orp::OrpPlugin;
use crate::playback::PlaybackPlugin;
use crate::tabs::{ActiveTab, TabWpm, WordsManager, TabsPlugin};

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
            .add_plugins((OrpPlugin, InputPlugin, PlaybackPlugin, TabsPlugin))
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
    /// Sets the initial timer for the current word when entering Playing state.
    fn on_start_playing(
        mut timer: ResMut<ReadingTimer>,
        active_tab: Single<(&TabWpm, &WordsManager), With<ActiveTab>>,
    ) {
        let (tab_wpm, words_mgr) = active_tab.into_inner();
        if let Some(word) = words_mgr.current_word() {
            let delay = Duration::from_millis(word.display_duration_ms(tab_wpm.0));
            timer.timer = Timer::new(delay, TimerMode::Once);
        }
    }

    /// Advances words when the per-word timer expires. Each word gets a fresh
    /// one-shot timer based on its display_duration_ms (varies by punctuation, length, etc.).
    fn tick(
        time: Res<Time>,
        mut timer: ResMut<ReadingTimer>,
        active_tab: Single<(&TabWpm, &mut WordsManager), With<ActiveTab>>,
        mut next_state: ResMut<NextState<ReadingState>>,
    ) {
        let (tab_wpm, mut words_mgr) = active_tab.into_inner();
        
        timer.timer.tick(time.delta());
        
        if timer.timer.just_finished() {
            if words_mgr.advance() {
                let word = words_mgr.current_word().unwrap();
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
