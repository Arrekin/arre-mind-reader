//! Playback command processing for reading state transitions.
//!
//! Centralizes playback control logic that can be triggered from UI or keyboard.

use bevy::prelude::*;

use crate::tabs::{ActiveTab, Content, TabWpm};
use crate::reader::{ReadingState, WordChanged};

pub struct PlaybackPlugin;
impl Plugin for PlaybackPlugin {
    fn build(&self, app: &mut App) {
        app
            .add_observer(PlaybackCommand::on_trigger)
            ;
    }
}

const WORD_SKIP_AMOUNT: usize = 5;

// ============================================================================
// Playback Commands
// ============================================================================

/// All playback actions, dispatched from keyboard input and UI controls.
/// Processed by a single observer that routes to state transitions and content mutations.
#[derive(Event)]
pub enum PlaybackCommand {
    TogglePlayPause,
    Restart,
    SkipForward(usize),
    SkipBackward(usize),
    AdjustWpm(i32),
}
impl PlaybackCommand {
    pub fn skip_forward() -> Self {
        Self::SkipForward(WORD_SKIP_AMOUNT)
    }

    pub fn skip_backward() -> Self {
        Self::SkipBackward(WORD_SKIP_AMOUNT)
    }
    /// Central command handler. Uses `Query` (not `Single`) for `active_tabs` because
    /// some commands (e.g. `Stop`) are valid even without an active reader tab.
    fn on_trigger(
        trigger: On<PlaybackCommand>,
        mut commands: Commands,
        current_state: Res<State<ReadingState>>,
        mut next_state: ResMut<NextState<ReadingState>>,
        mut active_tabs: Query<(&mut TabWpm, &mut Content), With<ActiveTab>>,
    ) {
        match trigger.event() {
            PlaybackCommand::TogglePlayPause => {
                match current_state.get() {
                    ReadingState::Playing => next_state.set(ReadingState::Paused),
                    _ => {
                        let can_play = active_tabs.single()
                            .is_ok_and(|(_, content)| content.has_words());
                        if can_play {
                            next_state.set(ReadingState::Playing);
                        }
                    }
                }
            }
            PlaybackCommand::Restart => {
                if let Ok((_, mut content)) = active_tabs.single_mut() {
                    content.restart();
                    commands.trigger(WordChanged);
                }
            }
            PlaybackCommand::SkipForward(amount) => {
                if let Ok((_, mut content)) = active_tabs.single_mut() {
                    content.skip_forward(*amount);
                    commands.trigger(WordChanged);
                }
            }
            PlaybackCommand::SkipBackward(amount) => {
                if let Ok((_, mut content)) = active_tabs.single_mut() {
                    content.skip_backward(*amount);
                    commands.trigger(WordChanged);
                }
            }
            PlaybackCommand::AdjustWpm(delta) => {
                if let Ok((mut tab_wpm, _)) = active_tabs.single_mut() {
                    let new_wpm = (tab_wpm.0 as i32 + delta)
                        .max(crate::reader::WPM_MIN as i32)
                        .min(crate::reader::WPM_MAX as i32);
                    tab_wpm.0 = new_wpm as u32;
                }
            }
        }
    }
}
