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
            .add_observer(WordChanged::on_trigger)
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
    fn on_start_playing(mut commands: Commands) {
        commands.trigger(WordChanged);
    }

    /// Advances words when the per-word timer expires, then signals WordChanged
    /// so the timer is reset for the next word by the observer.
    fn tick(
        time: Res<Time>,
        mut commands: Commands,
        mut timer: ResMut<ReadingTimer>,
        active_tab: Single<&mut WordsManager, With<ActiveTab>>,
        mut next_state: ResMut<NextState<ReadingState>>,
    ) {
        timer.timer.tick(time.delta());
        
        if timer.timer.just_finished() {
            let mut words_mgr = active_tab.into_inner();
            if words_mgr.advance() {
                commands.trigger(WordChanged);
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

/// Fired whenever the current word changes (advance, skip, restart, tab switch).
/// The observer resets ReadingTimer to the new word's display duration.
#[derive(Event)]
pub struct WordChanged;
impl WordChanged {
    fn on_trigger(
        _trigger: On<WordChanged>,
        mut timer: ResMut<ReadingTimer>,
        active_tabs: Query<(&TabWpm, &WordsManager), With<ActiveTab>>,
    ) {
        let Ok((wpm, words_mgr)) = active_tabs.single() else { return };
        if let Some(word) = words_mgr.current_word() {
            let delay = Duration::from_millis(word.display_duration_ms(wpm.0));
            timer.timer = Timer::new(delay, TimerMode::Once);
        }
    }
}
