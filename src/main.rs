//! Arre Mind Reader - A speed-reading application using RSVP (Rapid Serial Visual Presentation).
//!
//! Built with Bevy 0.18 game engine. Displays words one at a time with the Optical Recognition
//! Point (ORP) fixed at screen center for optimal reading speed.

use bevy::prelude::*;
use bevy_egui::EguiPlugin;

mod fonts;
mod input;
mod orp;
mod persistence;
mod reader;
mod state;
mod text;
mod timing;
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
            persistence::PersistencePlugin, 
            ui::UiPlugin
        ))
        .add_systems(Startup, setup)
        .run();
}

fn setup(mut commands: Commands) {
    commands.spawn(Camera2d);
}
