//! Playback command processing for reading state transitions.
//!
//! Centralizes playback control logic that can be triggered from UI or keyboard.

use bevy::prelude::*;

use crate::tabs::{ActiveTab, Content, TabWpm};
use crate::reader::ReadingState;

pub struct PlaybackPlugin;
impl Plugin for PlaybackPlugin {
    fn build(&self, app: &mut App) {
        app
            .add_observer(PlaybackCommand::on_trigger)
            ;
    }
}

// ============================================================================
// Playback Commands
// ============================================================================

/// All playback actions, dispatched from keyboard input and UI controls.
/// Processed by a single observer that routes to playback state and WPM updates.
#[derive(Event)]
pub enum PlaybackCommand {
    TogglePlayPause,
    AdjustWpm(i32),
}
impl PlaybackCommand {
    /// Central command handler. Uses `Query` (not `Single`) for `active_tabs` because
    /// some commands (e.g. `Stop`) are valid even without an active reader tab.
    fn on_trigger(
        trigger: On<PlaybackCommand>,
        current_state: Res<State<ReadingState>>,
        mut next_state: ResMut<NextState<ReadingState>>,
        mut active_tabs: Query<(&mut TabWpm, &Content), With<ActiveTab>>,
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


#[cfg(test)]
mod tests {
    use super::*;
    use crate::reader::{ReadingState, WPM_MAX, WPM_MIN};
    use crate::text::Word;

    fn make_test_app() -> App {
        let mut app = App::new();
        app
            .add_plugins(MinimalPlugins)
            .add_plugins(bevy::state::app::StatesPlugin)
            .init_state::<ReadingState>()
            .add_observer(PlaybackCommand::on_trigger)
            ;
        app
    }

    fn spawn_active_tab(app: &mut App, words: Vec<Word>, current_index: usize, wpm: u32) -> Entity {
        app.world_mut().spawn((
            ActiveTab,
            TabWpm(wpm),
            Content {
                content_cache_id: "test-cache".into(),
                words,
                current_index,
            },
        )).id()
    }

    #[test]
    fn toggle_play_pause_does_not_start_when_content_is_empty() {
        let mut app = make_test_app();
        spawn_active_tab(&mut app, Vec::new(), 0, 300);

        app.world_mut().trigger(PlaybackCommand::TogglePlayPause);
        app.update();

        assert_eq!(app.world().resource::<State<ReadingState>>().get(), &ReadingState::Idle);
    }

    #[test]
    fn toggle_play_pause_transitions_between_playing_and_paused() {
        let mut app = make_test_app();
        spawn_active_tab(&mut app, vec![Word::new("hello")], 0, 300);

        app.world_mut().trigger(PlaybackCommand::TogglePlayPause);
        app.update();
        assert_eq!(app.world().resource::<State<ReadingState>>().get(), &ReadingState::Playing);

        app.world_mut().trigger(PlaybackCommand::TogglePlayPause);
        app.update();
        assert_eq!(app.world().resource::<State<ReadingState>>().get(), &ReadingState::Paused);
    }

    #[test]
    fn adjust_wpm_clamps_to_limits() {
        let mut app = make_test_app();
        let active_tab_entity = spawn_active_tab(&mut app, vec![Word::new("hello")], 0, 300);

        app.world_mut().trigger(PlaybackCommand::AdjustWpm(10_000));
        let tab_wpm = app.world().entity(active_tab_entity).get::<TabWpm>()
            .expect("Active tab should have TabWpm component");
        assert_eq!(tab_wpm.0, WPM_MAX);

        app.world_mut().trigger(PlaybackCommand::AdjustWpm(-10_000));
        let tab_wpm = app.world().entity(active_tab_entity).get::<TabWpm>()
            .expect("Active tab should have TabWpm component");
        assert_eq!(tab_wpm.0, WPM_MIN);
    }

}
