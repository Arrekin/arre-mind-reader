use bevy::prelude::*;

use crate::state::OpenBooks;

pub struct SettingsPlugin;

impl Plugin for SettingsPlugin {
    fn build(&self, app: &mut App) {
        app.init_resource::<OpenBooks>();
    }
}
