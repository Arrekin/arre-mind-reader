use bevy::prelude::*;
use bevy_egui::EguiPlugin;

mod fonts;
mod reader;
mod settings;
mod state;
mod ui;

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
        .add_plugins((
            fonts::FontsPlugin, 
            reader::ReaderPlugin, 
            settings::SettingsPlugin, 
            ui::UiPlugin
        ))
        .add_systems(Startup, setup)
        .run();
}

fn setup(mut commands: Commands) {
    commands.spawn(Camera2d);
}
