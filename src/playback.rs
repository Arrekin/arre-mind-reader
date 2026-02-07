//! Playback command processing for reading state transitions.
//!
//! Centralizes playback control logic that can be triggered from UI or keyboard.

use bevy::prelude::*;

use crate::tabs::{ActiveTab, TabWpm, WordsManager};
use crate::reader::{ReadingState, WordChanged};

pub struct PlaybackPlugin;
impl Plugin for PlaybackPlugin {
    fn build(&self, app: &mut App) {
        app
            .add_message::<PlaybackCommand>()
            .add_systems(Update, PlaybackCommand::process)
            ;
    }
}

const WORD_SKIP_AMOUNT: usize = 5;

// ============================================================================
// Playback Commands
// ============================================================================

#[derive(Message)]
pub enum PlaybackCommand {
    TogglePlayPause,
    Stop,
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
    fn process(
        mut commands: Commands,
        mut events: MessageReader<PlaybackCommand>,
        current_state: Res<State<ReadingState>>,
        mut next_state: ResMut<NextState<ReadingState>>,
        mut active_tabs: Query<(&mut TabWpm, &mut WordsManager), With<ActiveTab>>,
    ) {
        for cmd in events.read() {
            match cmd {
                PlaybackCommand::TogglePlayPause => {
                    match current_state.get() {
                        ReadingState::Playing => next_state.set(ReadingState::Paused),
                        _ => {
                            let can_play = active_tabs.single()
                                .is_ok_and(|(_, words_mgr)| words_mgr.has_words());
                            if can_play {
                                next_state.set(ReadingState::Playing);
                            }
                        }
                    }
                }
                PlaybackCommand::Stop => {
                    next_state.set(ReadingState::Idle);
                }
                PlaybackCommand::Restart => {
                    if let Ok((_, mut words_mgr)) = active_tabs.single_mut() {
                        words_mgr.restart();
                        commands.trigger(WordChanged);
                    }
                }
                PlaybackCommand::SkipForward(amount) => {
                    if let Ok((_, mut words_mgr)) = active_tabs.single_mut() {
                        words_mgr.skip_forward(*amount);
                        commands.trigger(WordChanged);
                    }
                }
                PlaybackCommand::SkipBackward(amount) => {
                    if let Ok((_, mut words_mgr)) = active_tabs.single_mut() {
                        words_mgr.skip_backward(*amount);
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
}
