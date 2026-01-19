use bevy::prelude::*;
use bevy_egui::EguiPlugin;

mod reader;
mod settings;
mod state;
mod ui;

use reader::ReaderPlugin;
use settings::SettingsPlugin;
use ui::UiPlugin;

fn main() {
    App::new()
        .add_plugins(DefaultPlugins.set(WindowPlugin {
            primary_window: Some(Window {
                title: "Arre Mind Reader".into(),
                resolution: (1280, 720).into(),
                ..default()
            }),
            ..default()
        }))
        .add_plugins(EguiPlugin::default())
        .add_plugins((ReaderPlugin, SettingsPlugin, UiPlugin))
        .add_systems(Startup, setup)
        .run();
}

fn setup(mut commands: Commands) {
    commands.spawn(Camera2d);
}
