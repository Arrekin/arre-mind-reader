//! UI systems using bevy_egui.
//!
//! Provides tab bar, playback controls, settings panel, homepage tiles, and the new tab dialog.
//! UI components emit events/commands rather than directly mutating state.

mod tab_bar;
mod controls;
mod dialogs;
mod homepage;

use bevy::prelude::*;
use bevy_egui::EguiPrimaryContextPass;

pub use dialogs::{NewTabDialog, PendingFileLoad};

pub struct UiPlugin;
impl Plugin for UiPlugin {
    fn build(&self, app: &mut App) {
        app
            .init_resource::<NewTabDialog>()
            .init_resource::<PendingFileLoad>()
            .add_systems(Startup, homepage::HomepageTile::spawn)
            .add_systems(Update, dialogs::PendingFileLoad::poll)
            .add_systems(EguiPrimaryContextPass, (
                (tab_bar::tab_bar_system, controls::controls_system),
                dialogs::NewTabDialog::update.run_if(dialogs::NewTabDialog::is_open),
                (
                    homepage::HomepageTile::background,
                    homepage::AboutTile::update,
                    homepage::FontSettingsTile::update,
                    homepage::ShortcutsTile::update,
                    // homepage::StatsTile::update,
                    homepage::TipsTile::update,
                ).run_if(homepage::HomepageTile::is_active),
            ).chain())
            ;
    }
}
