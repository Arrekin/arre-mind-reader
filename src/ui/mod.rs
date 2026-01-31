//! UI systems using bevy_egui.
//!
//! Provides tab bar, playback controls, settings panel, and the new tab dialog.
//! UI components emit events/commands rather than directly mutating state.

mod tab_bar;
mod controls;
mod dialogs;

use bevy::prelude::*;
use bevy_egui::EguiPrimaryContextPass;

pub use dialogs::{NewTabDialog, PendingFileLoad};

pub struct UiPlugin;
impl Plugin for UiPlugin {
    fn build(&self, app: &mut App) {
        app
            .init_resource::<NewTabDialog>()
            .init_resource::<PendingFileLoad>()
            .add_systems(Update, dialogs::poll_file_load_task)
            .add_systems(EguiPrimaryContextPass, (
                tab_bar::tab_bar_system,
                controls::controls_system,
                dialogs::new_tab_dialog_system,
            ))
            ;
    }
}
