//! Settings persistence using RON format.
//!
//! Saves and loads tab state (open files, reading positions) to the user's config directory.
//! Uses debounced saving to avoid excessive disk writes.

use bevy::log::{debug, warn};
use bevy::prelude::*;
use std::path::PathBuf;

use crate::state::TabManager;

const TABS_FILE: &str = "tabs.ron";
const SAVE_DEBOUNCE_SECS: f32 = 5.0;

#[derive(Resource)]
struct TabSaveTimer {
    timer: Timer,
    pending: bool,
}

impl Default for TabSaveTimer {
    fn default() -> Self {
        Self {
            timer: Timer::from_seconds(SAVE_DEBOUNCE_SECS, TimerMode::Once),
            pending: false,
        }
    }
}

fn get_config_path() -> Option<PathBuf> {
    dirs::config_dir().map(|p| p.join("arre-mind-reader"))
}

fn load_tabs() -> TabManager {
    let Some(config_dir) = get_config_path() else {
        warn!("Could not determine config directory");
        return TabManager::default();
    };
    let path = config_dir.join(TABS_FILE);
    if !path.exists() {
        debug!("No saved tabs file found at {:?}", path);
        return TabManager::default();
    }
    match std::fs::read_to_string(&path) {
        Ok(content) => match ron::from_str::<TabManager>(&content) {
            Ok(tabs) => {
                debug!("Loaded tabs from {:?}", path);
                tabs
            }
            Err(e) => {
                warn!("Failed to parse tabs file: {}", e);
                TabManager::default()
            }
        },
        Err(e) => {
            warn!("Failed to read tabs file: {}", e);
            TabManager::default()
        }
    }
}

fn save_tabs(tabs: &TabManager) {
    let Some(config_dir) = get_config_path() else {
        warn!("Could not determine config directory for saving");
        return;
    };
    if let Err(e) = std::fs::create_dir_all(&config_dir) {
        warn!("Failed to create config directory: {}", e);
        return;
    }
    let path = config_dir.join(TABS_FILE);
    match ron::ser::to_string_pretty(tabs, ron::ser::PrettyConfig::default()) {
        Ok(content) => {
            if let Err(e) = std::fs::write(&path, content) {
                warn!("Failed to write tabs file: {}", e);
            } else {
                debug!("Saved tabs to {:?}", path);
            }
        }
        Err(e) => warn!("Failed to serialize tabs: {}", e),
    }
}

fn persist_tabs(
    tabs: Res<TabManager>,
    time: Res<Time>,
    mut save_timer: ResMut<TabSaveTimer>,
) {
    if tabs.is_changed() {
        save_timer.timer.reset();
        save_timer.pending = true;
    }
    
    if save_timer.pending {
        save_timer.timer.tick(time.delta());
        if save_timer.timer.just_finished() {
            save_tabs(&tabs);
            save_timer.pending = false;
        }
    }
}

fn save_tabs_on_exit(mut exit_messages: MessageReader<AppExit>, tabs: Res<TabManager>) {
    if exit_messages.read().next().is_some() {
        save_tabs(&tabs);
    }
}

pub struct SettingsPlugin;

impl Plugin for SettingsPlugin {
    fn build(&self, app: &mut App) {
        app.insert_resource(load_tabs())
            .init_resource::<TabSaveTimer>()
            .add_systems(PostUpdate, persist_tabs)
            .add_systems(Last, save_tabs_on_exit);
    }
}
