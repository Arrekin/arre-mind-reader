//! Reading state management and timing.

use std::time::Duration;
use bevy::prelude::*;

use crate::tabs::{ActiveTab, Content, TabWpm};

pub const WPM_DEFAULT: u32 = 300;
pub const WPM_MIN: u32 = 100;
pub const WPM_MAX: u32 = 1000;
pub const WPM_STEP: u32 = 50;
pub const FONT_SIZE_DEFAULT: f32 = 48.0;
pub const FONT_SIZE_MIN: f32 = 16.0;
pub const FONT_SIZE_MAX: f32 = 128.0;

pub struct ReaderPlugin;
impl Plugin for ReaderPlugin {
    fn build(&self, app: &mut App) {
        app.init_state::<ReadingState>()
            .init_resource::<ReadingTimer>()
            .add_systems(Update, ReadingTimer::tick.run_if(in_state(ReadingState::Playing)))
            .add_systems(OnEnter(ReadingState::Playing), ReadingState::on_start_playing)
            .add_observer(ReadingTimer::reset_on_word_changed)
            ;
    }
}


/// Playback state machine. Transitions driven by `PlaybackCommand` events.
#[derive(States, Debug, Clone, PartialEq, Eq, Hash, Default)]
pub enum ReadingState {
    #[default]
    Idle,
    Playing,
    Paused,
}
impl ReadingState {
    /// Fires `WordChanged` on play start so the timer is initialized
    /// for the current word (which may have been changed while paused/idle).
    fn on_start_playing(mut commands: Commands) {
        commands.trigger(WordChanged);
    }
}

/// Per-word countdown. Reset by the `WordChanged` observer to the current
/// word's display duration. When it expires, `tick` advances to the next word.
#[derive(Resource, Default)]
pub struct ReadingTimer {
    pub timer: Timer,
}
impl ReadingTimer {
    fn tick(
        mut commands: Commands,
        time: Res<Time>,
        mut timer: ResMut<ReadingTimer>,
    ) {
        timer.timer.tick(time.delta());
        if timer.timer.just_finished() {
            commands.trigger(ContentNavigate::Advance);
        }
    }
    fn reset_on_word_changed(
        _trigger: On<WordChanged>,
        mut timer: ResMut<ReadingTimer>,
        active_tab: Single<(&TabWpm, &Content), With<ActiveTab>>,
    ) {
        let (wpm, content) = active_tab.into_inner();
        if let Some(word) = content.current_word() {
            let delay = Duration::from_millis(word.display_duration_ms(wpm.0));
            timer.timer = Timer::new(delay, TimerMode::Once);
        }
    }
}

/// Content position changes from timer advance or user seek.
/// Observer handles the actual Content mutation and emits WordChanged.
#[derive(Event)]
pub enum ContentNavigate {
    Advance,
    Seek(usize),
    SkipForward(usize),
    SkipBackward(usize),
}

/// Fired after content navigation and tab switch to refresh timer and ORP display.
#[derive(Event)]
pub struct WordChanged;
